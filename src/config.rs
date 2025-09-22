use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::fileutils::resolve_relative;
use crate::noise::NoiseColor;
use crate::timeutils::DurationSeconds;
use crate::tts::run_piper;
use crate::utils::{ms_to_samples, secs_to_samples};
use log::{debug, info};

/// Defaults
const DEFAULT_SAMPLE_RATE: u32 = 48_000;
const DEFAULT_GAIN: f32 = 0.95;
const DEFAULT_FADE_MS: f32 = 50.0;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    Linear,
    Exp,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Optional overrides
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub gain: Option<f32>,
    #[serde(default)]
    pub fade_ms: Option<f32>,

    /// A path to the working directory where it caches the results of generated audio, or looks
    /// for audio file mixins
    #[serde(default)]
    pub audio_dir: Option<PathBuf>,
    #[serde(skip)]
    pub _audio_dir: PathBuf,

    /// A path to the directory with onnx model files and onnx.json configs
    #[serde(default)]
    pub model_dir: Option<PathBuf>,
    #[serde(skip)]
    pub _model_dir: PathBuf,

    /// The sequence of audio segments
    pub segments: Vec<Segment>,

    #[serde(skip)]
    pub _normalized: bool,
}

/// Default carrier tone should be a reasonable 200.0 Hertz.
fn default_carrier() -> f32 {
    200.0
}

fn default_tone_gain() -> f32 {
    1.0
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ToneSpec {
    #[serde(default = "default_tone_gain")]
    pub gain: f32,
    #[serde(default = "default_carrier")]
    pub carrier: f32,
    pub hz: f32,
    #[serde(default)]
    pub noise: Option<NoiseSpec>,
}

fn default_offset() -> DurationSeconds {
    DurationSeconds(0.0f32)
}

fn default_tts_gain() -> f32 {
    1.0f32
}

fn default_audio_gain() -> f32 {
    1.0f32
}

#[derive(Debug, Deserialize, Clone)]
pub struct TTSSpec {
    #[serde(default = "default_offset")]
    pub offset: DurationSeconds,
    #[serde(default = "default_tts_gain")]
    pub gain: f32,
    pub text: String,
    /// Key is used for caching. Otherwise, it'd calculate the sha256 hash of the
    /// model::config::text
    pub key: Option<String>,
    pub model: String,
    pub config: Option<String>,
    #[serde(skip)]
    _model_path: PathBuf,
    #[serde(skip)]
    _config_path: PathBuf,
    #[serde(skip)]
    pub _out_path: PathBuf,
}

impl TTSSpec {
    pub fn init_paths(&mut self, audio_dir: &Path, model_dir: &Path) -> std::io::Result<()> {
        self._model_path = model_dir.join(self.model.clone());
        self._config_path = model_dir.join(format!("{}.json", self.model.clone()));
        if let Some(config_str) = &self.config {
            self._config_path = model_dir.join(config_str);
        }

        self._model_path = std::fs::canonicalize(&self._model_path)?;
        self._config_path = std::fs::canonicalize(&self._config_path)?;
        let key = self.get_key();
        debug!("canonicalized model path and config path for key {}", key);

        self._out_path = audio_dir.join(format!("_tts_{}", key));
        Ok(())
    }

    pub fn generate(&self, piper_bin: Option<&str>, force: bool) -> std::io::Result<()> {
        let out_path = self._out_path.to_str().unwrap();
        if Path::new(out_path).exists() {
            if force {
                info!(
                    "{} already exists, but forcing regeneration due to --force|-f",
                    out_path
                );
            } else {
                debug!(
                    "{} already exists - skipping TTS generation and using old one. Delete it or update key if you want to regenerate.",
                    out_path
                );
                return Ok(());
            }
        } else {
            info!("generating TTS: {}", out_path);
        }

        let model_path = self._model_path.to_str().unwrap();
        let maybe_config_path = self._config_path.to_str();
        run_piper(
            piper_bin,
            &self.text,
            model_path,
            maybe_config_path,
            out_path,
        )
    }

    /// Get or calculate the key being used to cache the output file.
    /// This is calculated with:
    ///     sha256(abs_model_path . "::" . abs_config_path . "::" . trimmed_text_as_bytes)
    fn get_key(&self) -> String {
        if let Some(k) = &self.key {
            return k.trim().to_string().clone();
        }
        let mut hasher = Sha256::new();

        hasher.update(self._model_path.to_string_lossy().as_bytes());
        hasher.update("::");
        hasher.update(self._config_path.to_string_lossy().as_bytes());
        hasher.update("::");
        hasher.update(self.text.trim().as_bytes());

        // Finalize.
        let digest = hasher.finalize();
        hex::encode(digest)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioSpec {
    #[serde(default = "default_offset")]
    pub offset: DurationSeconds,
    #[serde(default = "default_audio_gain")]
    pub gain: f32,
    pub path: String,
    #[serde(skip)]
    pub _path: PathBuf,
}

impl AudioSpec {
    pub fn init_paths(&mut self, audio_dir: &Path) -> std::io::Result<()> {
        self._path = audio_dir.join(&self.path);

        self._path = std::fs::canonicalize(&self._path)?;
        debug!(
            "canonicalized path for file {} to {:?}",
            self.path, self._path
        );

        Ok(())
    }
}

fn default_noise_gain() -> f32 {
    0.0
}

/// Something to mix in over a Segment (eg: play audio or TTS)
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AudioMixin {
    File(AudioSpec),
    TTS(TTSSpec),
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct NoiseSpec {
    #[serde(default = "default_noise_gain")]
    pub gain: f32,
    pub color: NoiseColor,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Segment {
    /// Keep a steady tone for the duration `dur`.
    Tone {
        dur: DurationSeconds,
        carrier: f32,
        hz: f32,
        #[serde(default = "default_tone_gain")]
        gain: f32,
        #[serde(default)]
        noise: Option<NoiseSpec>,
        #[serde(default)]
        audio: Vec<AudioMixin>,
    },
    /// Transition from -> to across duration, with an optional curve.
    Transition {
        dur: DurationSeconds,
        from: ToneSpec,
        to: ToneSpec,
        #[serde(default)]
        curve: Option<Curve>,
        #[serde(default)]
        audio: Vec<AudioMixin>,
    },
}

#[derive(Debug)]
pub enum Chunk {
    Tone {
        samples: usize,
        spec: ToneSpec,
    },
    Transition {
        samples: usize,
        from: ToneSpec,
        to: ToneSpec,
        curve: Curve,
    },
}

impl Chunk {
    pub fn samples(&self) -> usize {
        match self {
            Chunk::Tone { samples, .. } => *samples,
            Chunk::Transition { samples, .. } => *samples,
        }
    }
}

impl Config {
    pub fn normalize_paths(&mut self, config_path: &Path) {
        let base = config_path.parent().unwrap_or_else(|| Path::new("."));

        self._audio_dir =
            resolve_relative(base, &self.audio_dir).unwrap_or_else(|| PathBuf::from("."));
        self._model_dir =
            resolve_relative(base, &self.model_dir).unwrap_or_else(|| PathBuf::from("."));

        self._normalized = true;
    }
    pub fn ms_to_samples(&self, ms: f32) -> usize {
        ms_to_samples(ms, self.get_sample_rate())
    }
    pub fn secs_to_samples(&self, secs: f32) -> usize {
        secs_to_samples(secs, self.get_sample_rate())
    }
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE)
    }
    pub fn get_gain(&self) -> f32 {
        self.gain.unwrap_or(DEFAULT_GAIN).clamp(0.0, 1.0)
    }
    pub fn get_fade_ms(&self) -> f32 {
        self.fade_ms.unwrap_or(DEFAULT_FADE_MS).max(0.0)
    }

    /// Build a flat plan of samples to render by iterating segments
    pub fn create_chunks(
        mut self,
        piper_bin: Option<&str>,
        force: bool,
    ) -> Result<Vec<Chunk>, std::io::Error> {
        let mut chunks: Vec<Chunk> = Vec::new();
        let sr = self.get_sample_rate();
        let model_dir = self._model_dir;
        let audio_dir = self._audio_dir;
        std::fs::create_dir_all(&audio_dir)?;
        for seg in self.segments.iter_mut() {
            match seg {
                Segment::Tone {
                    dur,
                    gain,
                    carrier,
                    hz,
                    noise,
                    audio,
                } => {
                    let total = secs_to_samples(dur.0, sr);
                    for mixin in audio.iter_mut() {
                        match mixin {
                            AudioMixin::File(audio_spec) => {
                                debug!("found audio spec {:?}", audio_spec);
                                audio_spec.init_paths(&audio_dir)?;
                            }
                            AudioMixin::TTS(tts_spec) => {
                                debug!("found tts spec {:?}", tts_spec);
                                tts_spec.init_paths(&audio_dir, &model_dir)?;
                                tts_spec.generate(piper_bin, force)?;
                            }
                        }
                    }
                    chunks.push(Chunk::Tone {
                        samples: total,
                        spec: ToneSpec {
                            carrier: *carrier,
                            hz: *hz,
                            gain: *gain,
                            noise: *noise,
                        },
                    });
                }
                Segment::Transition {
                    dur,
                    from,
                    to,
                    curve,
                    audio,
                } => {
                    let total = secs_to_samples(dur.0, sr);
                    for mixin in audio.iter_mut() {
                        match mixin {
                            AudioMixin::File(audio_spec) => {
                                debug!("found audio spec {:?}", audio_spec);
                                audio_spec.init_paths(&audio_dir)?;
                            }
                            AudioMixin::TTS(tts_spec) => {
                                debug!("found tts spec {:?}", tts_spec);
                                tts_spec.init_paths(&audio_dir, &model_dir)?;
                                tts_spec.generate(piper_bin, force)?;
                            }
                        }
                    }
                    chunks.push(Chunk::Transition {
                        samples: total,
                        from: *from,
                        to: *to,
                        curve: curve.unwrap_or(Curve::Linear),
                    });
                }
            }
        }
        for (i, chunk) in chunks.iter().enumerate() {
            debug!("Chunk {}: {:?}", i, chunk);
        }
        Ok(chunks)
    }
}

use crate::utils::{ms_to_samples, secs_to_samples};
use serde::Deserialize;
use std::path::PathBuf;

/// Defaults
const DEFAULT_SAMPLE_RATE: u32 = 48_000;
const DEFAULT_GAIN: f32 = 0.9;
const DEFAULT_FADE_MS: f32 = 50.0;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    Linear,
    Exp,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Output filename
    pub out: PathBuf,

    /// Optional overrides
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub gain: Option<f32>,
    #[serde(default)]
    pub fade_ms: Option<f32>,

    /// The sequence of audio segments
    pub segments: Vec<Segment>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ToneSpec {
    #[serde(default = "default_carrier")]
    pub carrier: f32,
    pub hz: f32,
}

/// Default carrier tone should be a reasonable 200.0 Hertz.
fn default_carrier() -> f32 {
    200.0
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Segment {
    /// Keep a steady tone for the duration `dur`.
    Tone { dur: f32, carrier: f32, hz: f32 },
    /// Transition from -> to across duration, with an optional curve.
    Transition {
        dur: f32,
        from: ToneSpec,
        to: ToneSpec,
        #[serde(default)]
        curve: Option<Curve>,
    },
}

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
    pub fn create_chunks(&self) -> Vec<Chunk> {
        let mut chunks: Vec<Chunk> = Vec::new();
        for seg in &self.segments {
            match seg {
                Segment::Tone { dur, carrier, hz } => {
                    let total = self.secs_to_samples(*dur);
                    chunks.push(Chunk::Tone {
                        samples: total,
                        spec: ToneSpec {
                            carrier: *carrier,
                            hz: *hz,
                        },
                    });
                }
                Segment::Transition {
                    dur,
                    from,
                    to,
                    curve,
                } => {
                    let total = self.secs_to_samples(*dur);
                    chunks.push(Chunk::Transition {
                        samples: total,
                        from: *from,
                        to: *to,
                        curve: curve.unwrap_or(Curve::Linear),
                    });
                }
            }
        }
        chunks
    }
}

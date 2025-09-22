use crate::config::{AudioSpec, TTSSpec};
use std::path::{Path, PathBuf};

use hound;

#[derive(Debug, Clone)]
pub struct Mixin {
    pub gain: f32,
    pub path: PathBuf,
    pub offset: f32,
}

impl From<TTSSpec> for Mixin {
    fn from(tts: TTSSpec) -> Self {
        Mixin {
            gain: tts.gain,
            path: tts._out_path,
            offset: tts.offset.0,
        }
    }
}

impl From<AudioSpec> for Mixin {
    fn from(audio: AudioSpec) -> Self {
        Mixin {
            gain: audio.gain,
            path: audio._path,
            offset: audio.offset.0,
        }
    }
}

/// Keeps the internal sample rate of the source wav.
pub fn load_wav_to_f32(path: &Path) -> std::io::Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path).map_err(|err| {
        std::io::Error::other(format!(
            "hound: {:?} - wav reader failed to read {:?}",
            err, path
        ))
    })?;
    let spec = reader.spec();
    let sr = spec.sample_rate;

    let samples: Vec<f32> = reader
        .samples::<i16>() // or f32 if file is float
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    Ok((samples, sr))
}

/// Resamples the input from an old to new sample rate.
pub fn resample_linear(input: &[f32], in_sr: u32, out_sr: u32) -> Vec<f32> {
    if in_sr == out_sr {
        return input.to_vec();
    }

    let ratio = out_sr as f64 / in_sr as f64;
    let out_len = (input.len() as f64 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let pos = i as f64 / ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f64;

        if idx + 1 < input.len() {
            let s0 = input[idx];
            let s1 = input[idx + 1];
            out.push(s0 + (s1 - s0) * frac as f32);
        } else {
            out.push(input[idx]);
        }
    }

    out
}

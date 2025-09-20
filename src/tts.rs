use piper_rs::{Piper, PiperConfig};
/// Text to speech using piper/piper-rs
/// See models in ./models directory, eg: models/en_US-amy-medium.onnx
/// Each should have its own onnx file and that file + ".json" as its config, which is expected
/// below.
use std::fs;
use std::path::Path;

use crate::sink::new_sink;

/// Generate speech from a string and write to a sink (FLAC is possible if you build with support).
/// File type is chosen by the out file extension.
pub fn text_to_sink(
    text: &str,
    model_path: &str,
    out: &str,
    config_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load Piper model and config.
    let config_path = config_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("{model_path}.json"));
    let model = Piper::load(Path::new(model_path), Path::new(&config_path))?;
    let sample_rate = model.config.sample_rate;
    let cfg = PiperConfig::default();

    // mono f32 samples
    let audio = model.synthesize(&text, &cfg)?;
    // The sample rate actually comes from the config. The TTS AI is literally trained with a
    // sample rate, so we have to use what it provides in the `$.audio.sample_rate` field.
    let mut sink = new_sink(out, sample_rate)?;
    for sample in audio {
        sink.write_sample(sample)?;
    }
    sink.finalize()?;
    Ok(())
}

use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Wav,
    Flac,
}

pub trait AudioSink {
    /// Write one interleaved stereo frame in range [-1.0, 1.0]
    fn write_frame(&mut self, left: f32, right: f32) -> Result<(), Box<dyn Error>>;
    /// Flush and finalize!
    fn finalize(self: Box<Self>) -> Result<(), Box<dyn Error>>;
}

/// For conversions from f32 to i16 for WAV
#[inline]
pub fn f32_to_i16(x: f32) -> i16 {
    let y = (x.clamp(-1.0, 1.0) * i16::MAX as f32).round();
    y as i16
}

pub use wav::WavSink;
mod wav;

pub use flac::FlacSink;
mod flac;

pub fn new_sink(out: &str, sample_rate: u32) -> Result<Box<dyn AudioSink>, Box<dyn Error>> {
    match detect_format_from_ext(out) {
        AudioFormat::Wav => WavSink::create(out, sample_rate),
        AudioFormat::Flac => FlacSink::create(out, sample_rate),
    }
}

/// Infer audio file format by file extension.
fn detect_format_from_ext(out: &str) -> AudioFormat {
    use std::path::Path;
    match Path::new(out)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "flac" => AudioFormat::Flac,
        "wav" => AudioFormat::Wav,
        // In the future, let's error out. But WAV for now.
        other => {
            eprintln!(
                "File extension was {:?}, writing WAV (only wav and flav supported).",
                other
            );
            AudioFormat::Wav
        }
    }
}

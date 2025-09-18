/// WAV writer
/// Uses stereo at 16-bit, and it MUST be stereo for binaural beats.
use std::path::PathBuf;

pub struct Writer {
    writer: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
}

impl Writer {
    pub fn new(out: &PathBuf, sample_rate: u32) -> hound::Result<Self> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer = hound::WavWriter::create(&out, spec)?;
        Ok(Self { writer })
    }

    pub fn write_sample(&mut self, sample: i16) -> hound::Result<()> {
        self.writer.write_sample(sample)
    }

    /// Finalize writing and close the file
    pub fn finalize(&mut self) -> hound::Result<()> {
        Ok(self.writer.flush()?)
    }
}

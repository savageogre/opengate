/// Default audio file writer.
use super::{AudioSink, f32_to_i16};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::{error::Error, fs::File, io::BufWriter, path::Path};

pub struct WavSink {
    writer: WavWriter<BufWriter<File>>,
}

impl WavSink {
    pub fn create(out: &str, sample_rate: u32) -> Result<Box<dyn AudioSink>, Box<dyn Error>> {
        let spec = WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let file = std::fs::File::create(Path::new(out))?;
        let buf = BufWriter::new(file);
        let writer = WavWriter::new(buf, spec)?;
        Ok(Box::new(WavSink { writer }))
    }
}

impl AudioSink for WavSink {
    fn write_frame(&mut self, l: f32, r: f32) -> Result<(), Box<dyn Error>> {
        self.writer.write_sample(f32_to_i16(l))?;
        self.writer.write_sample(f32_to_i16(r))?;
        Ok(())
    }
    fn finalize(self: Box<Self>) -> Result<(), Box<dyn Error>> {
        self.writer.finalize()?;
        Ok(())
    }
}

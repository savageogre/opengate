/// Flac generation is way messier here, and requires lib-flac:
///   Ubuntu/Debian: sudo apt install libflac-dev
///   Fedora/RHEL: sudo dnf install flac-devel
///   Arch: sudo pacman -S flac
///
/// This code is a bit messy with an unsafe block, but it works.
use super::{AudioSink, f32_to_i16};
use flac_bound::{FlacEncoder, WriteWrapper};
use std::{error::Error, fs::File, path::Path, ptr::NonNull};

pub struct FlacSink {
    file_ptr: NonNull<File>,
    wrapper_ptr: NonNull<WriteWrapper<'static>>,
    enc: Option<FlacEncoder<'static>>,
    buf: Vec<i32>,
    reclaimed: bool,
}

impl FlacSink {
    pub fn create(out: &str, sample_rate: u32) -> Result<Box<dyn AudioSink>, Box<dyn Error>> {
        let file_box = Box::new(File::create(Path::new(out))?);
        let file_ptr = Box::into_raw(file_box);
        let file_static: &'static mut File = unsafe { &mut *file_ptr };

        let wrapper_box = Box::new(WriteWrapper(file_static));
        let wrapper_ptr = Box::into_raw(wrapper_box);
        let wrapper_static: &'static mut WriteWrapper<'static> = unsafe { &mut *wrapper_ptr };

        // Build encoder will borrow the static lifetime here:
        let enc = FlacEncoder::new()
            .unwrap()
            .channels(2)
            .sample_rate(sample_rate)
            .bits_per_sample(16)
            .compression_level(5)
            .init_write(wrapper_static)
            .unwrap();

        Ok(Box::new(FlacSink {
            file_ptr: NonNull::new(file_ptr).unwrap(),
            wrapper_ptr: NonNull::new(wrapper_ptr).unwrap(),
            enc: Some(enc),
            buf: Vec::with_capacity(4096),
            reclaimed: false,
        }))
    }

    // SAFETY: We only call this after the encoder has been finished and won't touch the writer anymore.
    fn reclaim_leaks(&mut self) {
        if self.reclaimed {
            return;
        }
        unsafe {
            let _wrapper: Box<WriteWrapper<'static>> = Box::from_raw(self.wrapper_ptr.as_ptr());
            let _file: Box<File> = Box::from_raw(self.file_ptr.as_ptr());
        }
        self.reclaimed = true;
    }
}

impl AudioSink for FlacSink {
    fn write_frame(&mut self, l: f32, r: f32) -> Result<(), Box<dyn Error>> {
        self.buf.clear();
        self.buf.push(f32_to_i16(l) as i32);
        self.buf.push(f32_to_i16(r) as i32);

        let enc = self
            .enc
            .as_mut()
            .ok_or::<Box<dyn Error>>("flac: encoder not initialized".into())?;

        enc.process_interleaved(&self.buf, 1)
            .map_err(|_| "flac: process_interleaved failed".into())
    }

    fn finalize(mut self: Box<Self>) -> Result<(), Box<dyn Error>> {
        if let Some(enc) = self.enc.take() {
            let res = match enc.finish() {
                Ok(_) => Ok(()),
                Err(enc) => Err(format!("FLAC finalize failed: {:?}", enc.state()).into()),
            };
            self.reclaim_leaks();
            res
        } else {
            // Already finished or not initialized?
            self.reclaim_leaks();
            Ok(())
        }
    }
}

impl Drop for FlacSink {
    fn drop(&mut self) {
        // ...if the user forgot to call finalize:
        if let Some(enc) = self.enc.take() {
            let _ = enc.finish();
        }
        self.reclaim_leaks();
    }
}

use rustfft::{FftPlanner, num_complex::Complex};
use std::path::Path;

use hound;

fn read_wav(path: &Path) -> (Vec<f32>, Vec<f32>, u32) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();

    let mut left = Vec::new();
    let mut right = Vec::new();

    for chunk in samples.chunks(2) {
        if chunk.len() == 2 {
            left.push(chunk[0] as f32 / i32::MAX as f32);
            right.push(chunk[1] as f32 / i32::MAX as f32);
        }
    }

    (left, right, sample_rate)
}

fn dominant_freq(samples: &[f32], sample_rate: u32) -> f32 {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(samples.len());

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .map(|&s| Complex { re: s, im: 0.0 })
        .collect();
    fft.process(&mut buffer);

    let mut max_mag = 0.0;
    let mut max_idx = 0;
    for (i, c) in buffer.iter().enumerate().take(samples.len() / 2) {
        let mag = c.norm_sqr();
        if mag > max_mag {
            max_mag = mag;
            max_idx = i;
        }
    }

    max_idx as f32 * sample_rate as f32 / samples.len() as f32
}

/// Provided a path, analyze it and find the dominant frequencies in each channel.
pub fn analyze(path: &Path) {
    let (left, right, sr) = read_wav(path);

    let freq_left = dominant_freq(&left, sr);
    let freq_right = dominant_freq(&right, sr);

    println!("Left: {:.2} Hz - Right: {:.2} Hz", freq_left, freq_right);
    println!(
        "Binaural beat frequency: {:.2} Hz",
        (freq_left - freq_right).abs()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn make_sine_wave(freq: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
        let n_samples = (sample_rate as f32 * duration_secs) as usize;
        (0..n_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (TAU * freq * t).sin()
            })
            .collect()
    }

    #[test]
    fn test_dominant_freq_just_zero() {
        let samples = [0.0f32];
        assert_eq!(dominant_freq(&samples, 0), 0.0f32);
    }

    #[test]
    fn test_dominant_freq_pure_sine() {
        let sr = 8000;
        let freq = 440.0; // A4
        let samples = make_sine_wave(freq, sr, 1.0);
        let detected = dominant_freq(&samples, sr);
        assert!(
            (detected - freq).abs() < 5.0,
            "detected {detected}, expected {freq}"
        );
    }

    #[test]
    fn test_dominant_freq_high_frequency() {
        let sr = 44100;
        let freq = 10000.0;
        let samples = make_sine_wave(freq, sr, 0.1);
        let detected = dominant_freq(&samples, sr);
        assert!(
            (detected - freq).abs() < 50.0,
            "detected {detected}, expected {freq}"
        );
    }

    #[test]
    fn test_dominant_freq_two_tones_picks_strongest() {
        let sr = 8000;
        let f1 = make_sine_wave(440.0, sr, 1.0);
        let f2 = make_sine_wave(880.0, sr, 1.0);

        // Mix them, but bias amplitude toward 880 Hz
        let samples: Vec<f32> = f1
            .iter()
            .zip(f2.iter())
            .map(|(a, b)| a * 0.2 + b * 1.0)
            .collect();

        let detected = dominant_freq(&samples, sr);
        assert!(
            (detected - 880.0).abs() < 10.0,
            "detected {detected}, expected roughly 880 Hz"
        );
    }
}

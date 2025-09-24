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

use crate::config::{Chunk, Config, NoiseSpec};
use crate::noise::NoiseGenerator;
use crate::sink::new_sink;
use crate::utils::{apply_global_fade, ease, lerp};
/// Does the actual audio rendering magic.
use dasp::signal::Signal;
use std::f32::consts::TAU;

fn gain_or_zero(noise: &Option<NoiseSpec>) -> f32 {
    noise.as_ref().map(|ns| ns.gain).unwrap_or(0.0)
}

fn from_to_or_fallback<T: Clone>(from: &Option<T>, to: &Option<T>, t: f32) -> Option<T> {
    match (from, to) {
        (Some(left), Some(right)) => {
            if t < 0.5 {
                Some(left.clone())
            } else {
                Some(right.clone())
            }
        }
        (Some(left), _) => Some(left.clone()),
        (_, Some(right)) => Some(right.clone()),
        _ => None,
    }
}


/// Given a beat config and output path, write the file dynamically based on extension (WAV or
/// FLAC).
pub fn render(cfg: &Config, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = cfg.get_sample_rate();
    let gain = cfg.get_gain();
    let fade_ms = cfg.get_fade_ms();
    let dt = 1.0_f32 / sample_rate as f32;
    let chunks = cfg.create_chunks();

    let mut sink = new_sink(out, sample_rate)?;

    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let fade_len = cfg.ms_to_samples(fade_ms).min(total_samples / 2).max(1);

    // Phase accumulators
    let mut phase_l = 0.0_f32;
    let mut phase_r = 0.0_f32;

    let mut n_global = 0usize;
    for chunk in chunks {
        match chunk {
            Chunk::Tone {
                samples,
                spec,
            } => {
                let mut opt_ngen: Option<NoiseGenerator> = spec.noise.map(|ns| NoiseGenerator::new(ns.color));
                for _ in 0..samples {
                    let f_l = spec.carrier;
                    let f_r = spec.carrier + spec.hz;

                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    // Optionally, add noise.
                    if let Some(ref mut ngen) = opt_ngen {
                        left *= spec.gain;
                        right *= spec.gain;
                        // We can unwrap, because it only generates an ngen here if noise was
                        // Something.
                        let noise_val = ngen.next_sample() * spec.noise.unwrap().gain;
                        left += noise_val;
                        right += noise_val;
                    } else {
                        left *= spec.gain;
                        right *= spec.gain;
                    }

                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);
                    // We write this out as f32 [-1.0, 1.0] because the sinks handle quantization/encoding, depending
                    // on the file type.
                    sink.write_frame(left * gain, right * gain)?;
                    n_global += 1;
                }
            }
            Chunk::Transition {
                samples,
                from,
                to,
                curve,
            } => {
                let from_opt_ngen: Option<NoiseGenerator> = from.noise.map(|ns| NoiseGenerator::new(ns.color));
                let to_opt_ngen: Option<NoiseGenerator> = to.noise.map(|ns| NoiseGenerator::new(ns.color));

                let ramp = dasp::signal::from_iter((0..samples).map(move |n| {
                    let t = if samples <= 1 {
                        1.0
                    } else {
                        n as f32 / (samples - 1) as f32
                    };
                    ease(t, curve)
                }));

                let mut ramp_iter = ramp;
                for _ in 0..samples {
                    let t = ramp_iter.next();

                    let f_car = lerp(from.carrier, to.carrier, t);
                    let f_hz = lerp(from.hz, to.hz, t);
                    let f_gain = lerp(from.gain, to.gain, t);

                    let f_l = f_car;
                    let f_r = f_car + f_hz;

                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    // Optionally, add noise.
                    let mut opt_ngen = from_to_or_fallback(&from_opt_ngen, &to_opt_ngen, t);
                    if let Some(ref mut ngen) = opt_ngen {
                        left *= f_gain;
                        right *= f_gain;
                        let n_gain = lerp(gain_or_zero(&from.noise), gain_or_zero(&to.noise), t);
                        // We can unwrap, because it only generates an ngen here if noise was
                        // Something.
                        let noise_val = ngen.next_sample() * n_gain;
                        left += noise_val;
                        right += noise_val;
                    } else {
                        left *= f_gain;
                        right *= f_gain;
                    }

                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);
                    sink.write_frame(left * gain, right * gain)?;
                    n_global += 1;
                }
            }
        }
    }
    sink.finalize()?;
    Ok(())
}

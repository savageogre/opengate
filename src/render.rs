use crate::config::{Chunk, Config, NoiseSpec, ToneSpec};
use crate::noise::NoiseGenerator;
use crate::sink::new_sink;
use crate::utils::{apply_global_fade, ease, lerp, ms_to_samples};
/// Does the actual audio rendering magic.
use dasp::signal::Signal;
use std::f32::consts::TAU;

fn gain_or_zero(noise: &Option<NoiseSpec>) -> f32 {
    noise.as_ref().map(|ns| ns.gain).unwrap_or(0.0)
}

// This is a little tricky, because we want to be able to transition noise color and gain as well...
// How do you transition from pink noise at 1.0 gain to no noise definition? You lerp to zero.
// Or no noise to 1.0 pink noise? You have pink noise throughout, from 0 to 1.0.
fn from_to_or_fallback<'a, T>(
    from: &'a mut Option<T>,
    to: &'a mut Option<T>,
    t: f32,
) -> Option<&'a mut T> {
    match (from.as_mut(), to.as_mut()) {
        (Some(left), Some(right)) => {
            if t < 0.5 {
                Some(left)
            } else {
                Some(right)
            }
        }
        (Some(left), _) => Some(left),
        (_, Some(right)) => Some(right),
        _ => None,
    }
}

/// There's some complexity here, where we normalize tone and noise gain if they're both provided.
fn add_noise_and_fix_gain(
    left: &mut f32,
    right: &mut f32,
    spec: &ToneSpec,
    opt_ngen: &mut Option<NoiseGenerator>,
) {
    // If this is something, we have a noise generator and noise spec.
    if let Some(ngen) = opt_ngen.as_mut() {
        // Normalize gain.
        let mut n_gain = gain_or_zero(&spec.noise);
        let mut t_gain = spec.gain;
        let total_gain = spec.gain + n_gain;
        if total_gain > 1.0 {
            n_gain /= total_gain;
            t_gain /= total_gain;
        }
        *left *= t_gain;
        *right *= t_gain;
        let noise_val = ngen.next_sample() * n_gain;
        *left += noise_val;
        *right += noise_val;
    } else {
        *left *= spec.gain;
        *right *= spec.gain;
    }
}

fn add_noise_and_fix_gain_in_transition(
    left: &mut f32,
    right: &mut f32,
    from: &ToneSpec,
    to: &ToneSpec,
    opt_ngen: &mut Option<&mut NoiseGenerator>,
    t: f32,
) {
    let mut t_gain = lerp(from.gain, to.gain, t).clamp(0.0, 1.0);
    if let Some(ngen) = opt_ngen.as_mut() {
        let mut n_gain = lerp(gain_or_zero(&from.noise), gain_or_zero(&to.noise), t);
        let total_gain = t_gain + n_gain;
        if total_gain > 1.0 {
            t_gain /= total_gain;
            n_gain /= total_gain;
        }
        *left *= t_gain;
        *right *= t_gain;
        let noise_val = ngen.next_sample() * n_gain;
        *left += noise_val;
        *right += noise_val;
    } else {
        *left *= t_gain;
        *right *= t_gain;
    }
}

/// Given a beat config and output path, write the file dynamically based on extension (WAV or
/// FLAC).
pub fn render(
    cfg: Config,
    out: &str,
    piper_bin: Option<&str>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = cfg.get_sample_rate();
    let gain = cfg.get_gain();
    let fade_ms = cfg.get_fade_ms();
    let dt = 1.0_f32 / sample_rate as f32;
    let chunks = cfg.create_chunks(piper_bin, force)?;

    let mut sink = new_sink(out, sample_rate)?;

    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let fade_len = ms_to_samples(fade_ms, sample_rate)
        .min(total_samples / 2)
        .max(1);

    // Phase accumulators
    let mut phase_l = 0.0_f32;
    let mut phase_r = 0.0_f32;

    let mut n_global = 0usize;
    for chunk in chunks {
        match chunk {
            Chunk::Tone { samples, spec } => {
                let mut opt_ngen: Option<NoiseGenerator> =
                    spec.noise.map(|ns| NoiseGenerator::new(ns.color));
                for _ in 0..samples {
                    let f_l = spec.carrier;
                    let f_r = spec.carrier + spec.hz;

                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    add_noise_and_fix_gain(&mut left, &mut right, &spec, &mut opt_ngen);
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
                let mut from_ngen = from.noise.as_ref().map(|ns| NoiseGenerator::new(ns.color));
                let mut to_ngen = to.noise.as_ref().map(|ns| NoiseGenerator::new(ns.color));

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

                    let f_l = f_car;
                    let f_r = f_car + f_hz;

                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    // Optionally, add noise.
                    let mut opt_ngen = from_to_or_fallback(&mut from_ngen, &mut to_ngen, t);
                    add_noise_and_fix_gain_in_transition(
                        &mut left,
                        &mut right,
                        &from,
                        &to,
                        &mut opt_ngen,
                        t,
                    );
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

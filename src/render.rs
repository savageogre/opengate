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

fn render_stereo(
    chunks: Vec<Chunk>,
    sample_rate: u32,
    fade_ms: f32,
) -> Result<(Vec<f32>, Vec<f32>), Box<dyn std::error::Error>> {
    // Initialize the float vectors for both stereo tracks.
    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let mut lefts: Vec<f32> = Vec::with_capacity(total_samples);
    let mut rights: Vec<f32> = Vec::with_capacity(total_samples);

    let dt = 1.0_f32 / sample_rate as f32;
    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let fade_len = ms_to_samples(fade_ms, sample_rate)
        .min(total_samples / 2)
        .max(1);

    // Phase accumulators for binaural beats
    let mut phase_l = 0.0_f32;
    let mut phase_r = 0.0_f32;

    let mut n_global = 0usize;
    for chunk in chunks {
        let mut mixin_vec: Vec<f32> = vec![0.0; chunk.samples()];
        let mixin_dest: &mut [f32] = &mut mixin_vec;
        match chunk {
            Chunk::Tone {
                samples,
                spec,
                mixins,
            } => {
                for mixin in mixins {
                    mixin.render(mixin_dest, sample_rate)?;
                }
                let mut opt_ngen: Option<NoiseGenerator> =
                    spec.noise.map(|ns| NoiseGenerator::new(ns.color));
                #[allow(clippy::needless_range_loop)]
                for idx in 0..samples {
                    let f_l = spec.carrier;
                    let f_r = spec.carrier + spec.hz;

                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    add_noise_and_fix_gain(&mut left, &mut right, &spec, &mut opt_ngen);
                    left += mixin_dest[idx];
                    right += mixin_dest[idx];
                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);
                    // We write this out as f32 [-1.0, 1.0] because the sinks handle quantization/encoding, depending
                    // on the file type.
                    lefts.push(left);
                    rights.push(right);
                    n_global += 1;
                }
            }
            Chunk::Transition {
                samples,
                from,
                to,
                curve,
                mixins,
            } => {
                for mixin in mixins {
                    mixin.render(mixin_dest, sample_rate)?;
                }
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
                #[allow(clippy::needless_range_loop)]
                for idx in 0..samples {
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
                    left += mixin_dest[idx];
                    right += mixin_dest[idx];
                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);
                    lefts.push(left);
                    rights.push(right);
                    n_global += 1;
                }
            }
        }
    }
    Ok((lefts, rights))
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
    let fade_ms = cfg.get_fade_ms();
    let gain = cfg.get_gain();
    let chunks = cfg.create_chunks(piper_bin, force)?;

    let mut sink = new_sink(out, sample_rate)?;

    let (lefts, rights) = render_stereo(chunks, sample_rate, fade_ms)?;

    for idx in 0..lefts.len() {
        let left = lefts[idx];
        let right = rights[idx];
        sink.write_frame(left * gain, right * gain)?;
    }

    sink.finalize()?;
    Ok(())
}

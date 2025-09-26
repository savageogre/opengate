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

pub struct TrackCtx {
    pub left_samples: Vec<f32>,
    pub right_samples: Vec<f32>,
    pub sample_rate: u32,
    pub total_samples: usize,
    pub phase_l: f32,
    pub phase_r: f32,
    pub n_global: usize,
    pub fade_ms: f32,
    pub dt: f32,
}

impl TrackCtx {
    pub fn new(total_samples: usize, sample_rate: u32, fade_ms: f32) -> Self {
        let left_samples: Vec<f32> = Vec::with_capacity(total_samples);
        let right_samples: Vec<f32> = Vec::with_capacity(total_samples);

        // Phase accumulators for binaural beats
        let phase_l = 0.0_f32;
        let phase_r = 0.0_f32;

        let n_global = 0usize;
        let dt = 1.0_f32 / sample_rate as f32;

        Self {
            left_samples,
            right_samples,
            sample_rate,
            total_samples,
            phase_l,
            phase_r,
            n_global,
            fade_ms,
            dt,
        }
    }

    pub fn update_phases(&mut self, f_l: f32, f_r: f32) {
        self.phase_l = (self.phase_l + f_l * self.dt) % 1.0;
        self.phase_r = (self.phase_r + f_r * self.dt) % 1.0;
    }

    pub fn push(&mut self, left: f32, right: f32) {
        // We write this out as f32 [-1.0, 1.0] because the sinks handle quantization/encoding, depending
        // on the file type.
        self.left_samples.push(left);
        self.right_samples.push(right);
        self.n_global += 1;
    }

    pub fn render_chunk(&mut self, chunk: Chunk) -> Result<(), Box<dyn std::error::Error>> {
        let fade_len = ms_to_samples(self.fade_ms, self.sample_rate)
            .min(self.total_samples / 2)
            .max(1);
        let mut mixin_vec: Vec<f32> = vec![0.0; chunk.samples()];
        let mixin_dest: &mut [f32] = &mut mixin_vec;
        match chunk {
            Chunk::Tone {
                samples,
                spec,
                mixins,
            } => {
                for mixin in mixins {
                    mixin.render(mixin_dest, self.sample_rate)?;
                }
                let mut opt_ngen: Option<NoiseGenerator> =
                    spec.noise.map(|ns| NoiseGenerator::new(ns.color));
                #[allow(clippy::needless_range_loop)]
                for idx in 0..samples {
                    self.update_phases(spec.carrier, spec.carrier + spec.hz);

                    let (mut left, mut right) =
                        ((TAU * self.phase_l).sin(), (TAU * self.phase_r).sin());

                    add_noise_and_fix_gain(&mut left, &mut right, &spec, &mut opt_ngen);
                    left += mixin_dest[idx];
                    right += mixin_dest[idx];
                    apply_global_fade(
                        self.n_global,
                        self.total_samples,
                        fade_len,
                        &mut left,
                        &mut right,
                    );
                    self.push(left, right);
                }
                Ok(())
            }
            Chunk::Transition {
                samples,
                from,
                to,
                curve,
                mixins,
            } => {
                for mixin in mixins {
                    mixin.render(mixin_dest, self.sample_rate)?;
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

                    self.update_phases(f_car, f_car + f_hz);

                    let (mut left, mut right) =
                        ((TAU * self.phase_l).sin(), (TAU * self.phase_r).sin());

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
                    apply_global_fade(
                        self.n_global,
                        self.total_samples,
                        fade_len,
                        &mut left,
                        &mut right,
                    );
                    self.push(left, right);
                }
                Ok(())
            }
        }
    }

    pub fn render_chunks(&mut self, chunks: Vec<Chunk>) -> Result<(), Box<dyn std::error::Error>> {
        for chunk in chunks {
            self.render_chunk(chunk)?;
        }
        Ok(())
    }

    pub fn iter_samples(&self) -> impl Iterator<Item = (usize, f32, f32)> + '_ {
        self.left_samples
            .iter()
            .zip(self.right_samples.iter())
            // Stop at n_global if it's less than the length.
            .take(self.n_global)
            .enumerate()
            .map(|(i, (&left, &right))| (i, left, right))
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
    let fade_ms = cfg.get_fade_ms();
    let gain = cfg.get_gain();
    let chunks = cfg.create_chunks(piper_bin, force)?;

    let mut sink = new_sink(out, sample_rate)?;

    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let mut track_ctx = TrackCtx::new(total_samples, sample_rate, fade_ms);
    track_ctx.render_chunks(chunks)?;

    for (_, left, right) in track_ctx.iter_samples() {
        sink.write_frame(left * gain, right * gain)?;
    }

    sink.finalize()?;
    Ok(())
}

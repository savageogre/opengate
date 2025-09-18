mod config;
mod utils;

use dasp::signal::Signal;
use hound;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;
use clap::Parser;

use crate::config::{Chunk, Config, Curve, Segment, ToneSpec};
use crate::utils::{apply_global_fade, ease, secs_to_samples, ms_to_samples, lerp};

/// Defaults
const DEFAULT_SAMPLE_RATE: u32 = 48_000;
const DEFAULT_GAIN: f32 = 0.9;
const DEFAULT_FADE_MS: f32 = 50.0;


#[derive(Parser, Debug)]
#[command(
    author = "Savage Ogre",
    version = "0.1.0",
    about = "generate binaural beats for meditative purposes"
)]
struct Args {
    /// YAML configuration file
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cfg_text = fs::read_to_string(&args.config)?;
    let cfg: Config = serde_yaml::from_str(&cfg_text)?;

    let sample_rate = cfg.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
    let gain = cfg.gain.unwrap_or(DEFAULT_GAIN).clamp(0.0, 1.0);
    let fade_ms = cfg.fade_ms.unwrap_or(DEFAULT_FADE_MS).max(0.0);
    let dt = 1.0_f32 / sample_rate as f32;

    // Build a flat plan of samples to render by iterating segments
    let mut chunks: Vec<Chunk> = Vec::new();
    for seg in &cfg.segments {
        match seg {
            Segment::Tone { dur, carrier, hz } => {
                let total = secs_to_samples(*dur, sample_rate);
                chunks.push(Chunk::Tone {
                    samples: total,
                    spec: ToneSpec {
                        carrier: *carrier,
                        hz: *hz,
                    },
                });
            }
            Segment::Transition {
                dur,
                from,
                to,
                curve,
            } => {
                let total = secs_to_samples(*dur, sample_rate);
                chunks.push(Chunk::Transition {
                    samples: total,
                    from: *from,
                    to: *to,
                    curve: curve.unwrap_or(Curve::Linear),
                });
            }
        }
    }

    // Total length for global fade in/out
    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let fade_len = ms_to_samples(fade_ms, sample_rate)
        .min(total_samples / 2)
        .max(1);

    // WAV writer: stereo, 16-bit
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&cfg.out, spec)?;

    // Phase accumulators
    let mut phase_l = 0.0_f32;
    let mut phase_r = 0.0_f32;

    // Render
    let mut n_global = 0usize;
    for chunk in chunks {
        match chunk {
            Chunk::Tone { samples, spec } => {
                for _ in 0..samples {
                    // fixed frequencies per sample
                    let f_l = spec.carrier;
                    let f_r = spec.carrier + spec.hz;

                    // integrate phase
                    phase_l = (phase_l + f_l * dt) % 1.0;
                    phase_r = (phase_r + f_r * dt) % 1.0;

                    // sample
                    let (mut left, mut right) = ((TAU * phase_l).sin(), (TAU * phase_r).sin());

                    // global fade in/out to avoid clicks at file edges
                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);

                    // headroom
                    let li = (left * gain * i16::MAX as f32) as i16;
                    let ri = (right * gain * i16::MAX as f32) as i16;

                    writer.write_sample(li)?;
                    writer.write_sample(ri)?;
                    n_global += 1;
                }
            }
            Chunk::Transition {
                samples,
                from,
                to,
                curve,
            } => {
                // build simple ramp Signal for convenience (dasp)
                let ramp = dasp::signal::from_iter((0..samples).map(move |n| {
                    // normalized time in [0,1]
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

                    apply_global_fade(n_global, total_samples, fade_len, &mut left, &mut right);

                    let li = (left * gain * i16::MAX as f32) as i16;
                    let ri = (right * gain * i16::MAX as f32) as i16;

                    writer.write_sample(li)?;
                    writer.write_sample(ri)?;
                    n_global += 1;
                }
            }
        }
    }

    writer.finalize()?;
    println!("Wrote beats to: {:?}", &cfg.out);
    Ok(())
}

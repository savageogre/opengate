mod config;
mod utils;
mod writer;

use clap::Parser;
use dasp::signal::Signal;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

use crate::config::{Chunk, Config};
use crate::utils::{apply_global_fade, ease, lerp};
use crate::writer::Writer;

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

    let gain = cfg.get_gain();
    let fade_ms = cfg.get_fade_ms();
    let dt = 1.0_f32 / cfg.get_sample_rate() as f32;
    let chunks = cfg.create_chunks();

    // Total length for global fade in/out
    let total_samples: usize = chunks.iter().map(|c| c.samples()).sum();
    let fade_len = cfg.ms_to_samples(fade_ms).min(total_samples / 2).max(1);

    let writer = &mut Writer::new(&cfg.out, cfg.get_sample_rate())?;

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

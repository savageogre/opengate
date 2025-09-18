use clap::Parser;
use dasp::signal::Signal;
use hound;
use serde::Deserialize;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Deserialize)]
struct Config {
    /// Output filename
    out: PathBuf,

    /// Optional overrides
    #[serde(default)]
    sample_rate: Option<u32>,
    #[serde(default)]
    gain: Option<f32>,
    #[serde(default)]
    fade_ms: Option<f32>,

    /// The sequence of audio segments
    segments: Vec<Segment>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct ToneSpec {
    #[serde(default = "default_carrier")]
    carrier: f32,
    hz: f32,
}

/// Default carrier tone should be a reasonable 200.0 Hertz.
fn default_carrier() -> f32 {
    200.0
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Segment {
    /// Keep a steady tone for the duration `dur`.
    Tone { dur: f32, carrier: f32, hz: f32 },
    /// Transition from -> to across duration, with an optional curve.
    Transition {
        dur: f32,
        from: ToneSpec,
        to: ToneSpec,
        #[serde(default)]
        curve: Option<Curve>,
    },
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Curve {
    Linear,
    Exp,
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

enum Chunk {
    Tone {
        samples: usize,
        spec: ToneSpec,
    },
    Transition {
        samples: usize,
        from: ToneSpec,
        to: ToneSpec,
        curve: Curve,
    },
}

impl Chunk {
    fn samples(&self) -> usize {
        match self {
            Chunk::Tone { samples, .. } => *samples,
            Chunk::Transition { samples, .. } => *samples,
        }
    }
}

#[inline]
fn secs_to_samples(secs: f32, sr: u32) -> usize {
    ((secs.max(0.0)) * sr as f32).round() as usize
}

#[inline]
fn ms_to_samples(ms: f32, sr: u32) -> usize {
    secs_to_samples(ms / 1000.0, sr)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn ease(t: f32, curve: Curve) -> f32 {
    let x = t.clamp(0.0, 1.0);
    match curve {
        Curve::Linear => x,
        // Exponential-ish ease (smooth start/end): y = (e^(k x) - 1) / (e^k - 1)
        // with k ~ 4.0 for a noticeable curve.
        Curve::Exp => {
            let k = 4.0_f32;
            ((k * x).exp() - 1.0) / (k.exp() - 1.0)
        }
    }
}

// Apply a quick global fade in/out to avoid clicks at file boundaries.
fn apply_global_fade(n: usize, total: usize, fade_len: usize, left: &mut f32, right: &mut f32) {
    if n < fade_len {
        let g = n as f32 / fade_len as f32;
        *left *= g;
        *right *= g;
    } else if n + fade_len >= total {
        let remain = total - n;
        let g = (remain as f32 / fade_len as f32).clamp(0.0, 1.0);
        *left *= g;
        *right *= g;
    }
}

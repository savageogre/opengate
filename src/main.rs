use clap::Parser;
use dasp::signal::{self, Signal};
use hound;
/// OpenGate
///
/// This is free and open-source software to generate binaural beats for personal meditative
/// purposes.
use std::f32::consts::TAU;
use std::path::PathBuf;

const SAMPLE_RATE: u32 = 48_000;
#[derive(Parser, Debug)]
#[command(
    author = "Savage Ogre",
    version = "0.1.0",
    about = "generate binaural beats for meditative purposes"
)]
struct Args {
    #[arg(
        short,
        long = "out",
        default_value = "opengate.wav",
        help = "Output filename"
    )]
    out: PathBuf,

    // First beat
    #[arg(long = "dur0", default_value_t = 10.0, help = "Duration of first beat")]
    dur0: f32,
    #[arg(
        long = "car0",
        default_value_t = 200.0,
        help = "Carrier frequency of first beat"
    )]
    car0: f32,
    #[arg(long = "hz0", default_value_t = 7.0, help = "Hertz of first beat")]
    hz0: f32,

    // Between beats
    #[arg(
        long = "tdur0",
        default_value_t = 10.0,
        help = "Duration of the transition from first to second"
    )]
    tdur0: f32,

    // Second beat
    #[arg(
        long = "dur1",
        default_value_t = 10.0,
        help = "Duration of second beat"
    )]
    dur1: f32,
    #[arg(
        long = "car1",
        default_value_t = 100.0,
        help = "Carrier frequency of second beat"
    )]
    car1: f32,
    #[arg(long = "hz1", default_value_t = 3.875, help = "Hertz of second beat")]
    hz1: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let total_duration: f32 = args.dur0 + args.tdur0 + args.dur1;

    let f_l0 = args.car0;
    let f_r0 = args.car0 + args.hz0;

    let f_l1 = args.car1;
    let f_r1 = args.car1 + args.hz1;

    let total_samples = (total_duration * SAMPLE_RATE as f32) as usize;

    // TODO: this is just running for the whole thing, need to do entrainment stuff and use
    // transition.
    // Linear ramps for frequency over time using dasp_signal
    let ramp = |start: f32, end: f32| {
        signal::from_iter((0..total_samples).map(move |n| {
            let t = n as f32 / (total_samples.saturating_sub(1).max(1) as f32);
            start + (end - start) * t
        }))
    };

    let mut freq_l = ramp(f_l0, f_l1);
    let mut freq_r = ramp(f_r0, f_r1);

    // Phase accumulators
    let mut phase_l = 0.0_f32;
    let mut phase_r = 0.0_f32;
    let dt = 1.0_f32 / SAMPLE_RATE as f32;

    // WAV writer using 16-bit stereo (it MUST be stereo to be binaural beats).
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args.out.clone(), spec)?;

    // A simple fade in/out to avoid clicks - 0.05 being 50 ms.
    let fade_len = (0.05 * SAMPLE_RATE as f32) as usize;

    for n in 0..total_samples {
        let f_l = freq_l.next();
        let f_r = freq_r.next();

        phase_l = (phase_l + f_l * dt) % 1.0;
        phase_r = (phase_r + f_r * dt) % 1.0;

        let mut left = (TAU * phase_l).sin();
        let mut right = (TAU * phase_r).sin();

        // Apply quick fade at start and end.
        if n < fade_len {
            let g = n as f32 / fade_len as f32;
            left *= g;
            right *= g;
        } else if n > total_samples - fade_len {
            let g = (total_samples - n) as f32 / fade_len as f32;
            left *= g;
            right *= g;
        }

        // Reduce amplitude to avoid clipping for layering noise down the road.
        let amp = 0.9_f32;

        // Converting it to i16 with headroom.
        let li = (left * amp * i16::MAX as f32) as i16;
        let ri = (right * amp * i16::MAX as f32) as i16;

        writer.write_sample(li)?;
        writer.write_sample(ri)?;
    }

    writer.finalize()?;
    println!("Wrote beats to: {:?}", args.out);
    Ok(())
}

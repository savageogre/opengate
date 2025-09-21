use clap::Parser;
use std::fs;

use opengate::tts::run_piper;

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "piper",
        help = "optional path to piper binary if it's not in your path"
    )]
    piper_bin: Option<String>,

    #[arg(short, long, help = "path to .onnx file")]
    model: String,

    #[arg(
        short,
        long,
        help = "optional path to custom config, though it will choose $model.json by default"
    )]
    config: Option<String>,

    #[arg(short, long, help = "text file to turn into speech")]
    input: String,

    #[arg(short, long, default_value = "opengate-tts.wav", help = "output file")]
    out: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let text = fs::read_to_string(&args.input)?;

    run_piper(
        args.piper_bin.as_deref(),
        &text,
        &args.model,
        args.config.as_deref(),
        &args.out,
    )?;

    println!("TTS wrote: {}", args.out);
    Ok(())
}

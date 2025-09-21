use clap::Parser;
use log::info;
use std::fs;

use opengate::logger;
use opengate::tts::run_piper;

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "piper",
        help = "optional path to piper binary if it's not in your $PATH"
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

    #[arg(short = 'v', long = "verbose", help = "verbose level logging")]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logger::init(args.verbose);

    let text = fs::read_to_string(&args.input)?;

    run_piper(
        args.piper_bin.as_deref(),
        &text,
        &args.model,
        args.config.as_deref(),
        &args.out,
    )?;

    info!("TTS wrote: {}", args.out);
    Ok(())
}

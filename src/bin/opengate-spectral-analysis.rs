use clap::Parser;
use log::info;
use std::path::PathBuf;

use opengate::{analysis, logger};

#[derive(Parser, Debug)]
#[command(author, version, about = "perform FFT analysis on audio files")]
struct Args {
    #[arg(short = 'v', long = "verbose", help = "verbose level logging")]
    verbose: bool,

    #[arg(help = "path to audio file (only supports WAV file type for now)")]
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logger::init(args.verbose);
    let path = args.path;
    if !path.exists() {
        panic!("path does not exist");
    }
    info!("Processing file: {}", path.display());
    analysis::analyze(&path);
    Ok(())
}

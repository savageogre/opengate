mod config;
mod noise;
mod render;
mod sink;
mod utils;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::render::render;

#[derive(Parser, Debug)]
#[command(
    author = "Savage Ogre",
    version,
    about = "generate binaural beats for meditative purposes"
)]
struct Args {
    /// YAML configuration file
    config: PathBuf,
    #[arg(
        short,
        long,
        default_value = "opengate.wav",
        help = "output file, supporting wav or flac"
    )]
    out: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cfg_text = fs::read_to_string(&args.config)?;
    let cfg: Config = serde_yaml::from_str(&cfg_text)?;
    render(&cfg, &args.out)?;
    println!("Wrote beats to: {:?}", &args.out);
    Ok(())
}

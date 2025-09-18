mod config;
mod render;
mod utils;
mod writer;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::render::render;

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
    match render(&cfg) {
        Ok(()) => {
            println!("Wrote beats to: {:?}", &cfg.out);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

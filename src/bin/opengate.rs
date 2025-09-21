use clap::Parser;
use serde_yaml::Value;
use std::fs;
use std::path::PathBuf;
use yaml_merge_keys::merge_keys_serde;

use opengate::config::Config;
use opengate::render::render;

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
    let value: Value = serde_yaml::from_str(&cfg_text)?;
    let merged = merge_keys_serde(value)?;
    let cfg: Config = serde_yaml::from_value(merged)?;
    render(&cfg, &args.out)?;
    println!("Wrote beats to: {:?}", &args.out);
    Ok(())
}

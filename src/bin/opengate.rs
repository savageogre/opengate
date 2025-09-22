use clap::Parser;
use log::info;
use serde_yaml::Value;
use std::fs;
use std::path::PathBuf;
use yaml_merge_keys::merge_keys_serde;

use opengate::config::Config;
use opengate::logger;
use opengate::render::render;

#[derive(Parser, Debug)]
#[command(
    author = "Savage Ogre",
    version,
    about = "generate binaural beats for meditative purposes"
)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "piper",
        help = "optional path to piper binary if it's not in your $PATH"
    )]
    piper_bin: Option<String>,

    #[arg(
        short = 'f',
        long = "force",
        help = "force regeneration of audio files even if they exist, for example, if you update your piper model"
    )]
    force: bool,

    /// YAML configuration file
    config: PathBuf,

    #[arg(
        short,
        long,
        default_value = "opengate.wav",
        help = "output file, supporting wav or flac"
    )]
    out: String,

    #[arg(short = 'v', long = "verbose", help = "verbose level logging")]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logger::init(args.verbose);
    let cfg_text = fs::read_to_string(&args.config)?;
    let value: Value = serde_yaml::from_str(&cfg_text)?;
    let merged = merge_keys_serde(value)?;
    let mut cfg: Config = serde_yaml::from_value(merged)?;
    // This *MUST* run before render because audio and tts specs func init_paths uses the calculated paths.
    cfg.normalize_paths(&args.config);
    render(cfg, &args.out, args.piper_bin.as_deref(), args.force)?;
    info!("Wrote beats to: {:?}", &args.out);
    Ok(())
}

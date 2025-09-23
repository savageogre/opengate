use std::fs;
use std::path::Path;

use clap::Parser;
use log::info;
use reqwest::Client;

use opengate::{logger, sysconfig};

#[derive(Debug)]
struct Model {
    shortlang: String,
    lang: String,
    name: String,
    size: String,
}

impl Model {
    fn new(shortlang: &str, lang: &str, name: &str, size: &str) -> Self {
        Self {
            shortlang: shortlang.to_string(),
            lang: lang.to_string(),
            name: name.to_string(),
            size: size.to_string(),
        }
    }

    fn init_all() -> Vec<Self> {
        vec![
            Self::new("en", "en_US", "kristin", "medium"),
            Self::new("en", "en_US", "amy", "medium"),
            Self::new("en", "en_US", "reza_ibrahim", "medium"),
            Self::new("en", "en_US", "ryan", "high"),
            Self::new("en", "en_US", "libritts", "high"),
        ]
    }

    fn base_url(&self) -> String {
        format!(
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/{}/{}/{}/{}/{}-{}-{}",
            self.shortlang, self.lang, self.name, self.size, self.lang, self.name, self.size
        )
    }

    async fn download(&self, client: &Client, out_dir: &Path) -> reqwest::Result<()> {
        let base = self.base_url();

        fs::create_dir_all(out_dir).unwrap();

        let onnx_url = format!("{}.onnx", base);
        let onnx_path = out_dir.join(format!("{}-{}-{}.onnx", self.lang, self.name, self.size));
        let onnx_bytes = client.get(&onnx_url).send().await?.bytes().await?;
        info!("Downloading: {}", onnx_url);
        fs::write(&onnx_path, &onnx_bytes).unwrap();

        let json_url = format!("{}.onnx.json", base);
        let json_path = out_dir.join(format!(
            "{}-{}-{}.onnx.json",
            self.lang, self.name, self.size
        ));
        let json_bytes = client.get(&json_url).send().await?.bytes().await?;
        info!("Downloading: {}", json_url);
        fs::write(&json_path, &json_bytes).unwrap();

        info!("Downloaded {} and JSON to {:?}", self.name, out_dir);
        Ok(())
    }

    async fn download_one(
        out_dir: &Path,
        short_lang: &str,
        lang: &str,
        name: &str,
        size: &str,
    ) -> reqwest::Result<()> {
        let client = Client::new();
        let model = Self::new(short_lang, lang, name, size);
        model.download(&client, out_dir).await?;
        Ok(())
    }

    async fn download_all(out_dir: &Path) -> reqwest::Result<()> {
        let client = Client::new();
        for m in Self::init_all() {
            m.download(&client, out_dir).await?;
        }
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "download piper Text-To-Speech models and configs to system model dir"
)]
struct Args {
    #[arg(short = 'v', long = "verbose", help = "verbose level logging")]
    verbose: bool,

    #[arg(short, long = "all", help = "download all models")]
    all: bool,

    #[arg(
        long = "short-lang",
        default_value = "en",
        help = "short language alias"
    )]
    short_lang: String,
    #[arg(long = "lang", default_value = "en_US", help = "full language alias")]
    lang: String,
    #[arg(long = "name", default_value = "libritts", help = "language name")]
    name: String,
    #[arg(
        long = "size",
        default_value = "high",
        help = "language size (low/medium/high)"
    )]
    size: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logger::init(args.verbose);

    let models_dir = sysconfig::get_models_dir()?;
    info!("System models directory is at: {}", models_dir.display());

    if args.all {
        Model::download_all(&models_dir).await?;
    } else {
        Model::download_one(
            &models_dir,
            &args.short_lang,
            &args.lang,
            &args.name,
            &args.size,
        )
        .await?;
    }

    Ok(())
}

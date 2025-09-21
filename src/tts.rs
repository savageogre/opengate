use std::io::Write;
/// Text to speech using piper/piper-rs
/// See models in ./text_to_speech/models directory, eg: en_US-amy-medium.onnx
/// Each should have its own onnx file and that file + ".json" as its config, which is expected
/// below.
/// I tried to integrate piper, but the ort dependency was killing me. It's much easier to just run
/// their binary - and that source and linux x86/64 binary is included in the repo in
/// ./text_to_speech
/// Piper needs to be in your path or the path to the binary passed in manually.
use std::process::{Command, Stdio};

pub fn run_piper(
    piper_bin: Option<&str>,
    text: &str,
    model_path: &str,
    config_path: Option<&str>,
    output_path: &str,
) -> std::io::Result<()> {
    let piper_bin = piper_bin.unwrap_or("piper");
    let config_path: String = config_path
        .map(|c| c.to_string())
        .unwrap_or_else(|| format!("{}.json", model_path));
    let mut child = Command::new(piper_bin)
        .arg("-m")
        .arg(model_path)
        .arg("-c")
        .arg(&config_path)
        .arg("-f")
        .arg(output_path)
        .stdin(Stdio::piped())
        .spawn()?;

    // Write text to Piper's stdin.
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    }

    // Wait for Piper to finish.
    let status = child.wait()?;
    if !status.success() {
        eprintln!("Piper exited with status: {:?}", status);
    }

    Ok(())
}

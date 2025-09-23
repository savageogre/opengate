use directories::ProjectDirs;
use log::info;
use std::fs;
use std::path::PathBuf;

fn get_sysconfig_dir() -> std::io::Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("org", "savageogre", "opengate") {
        let cache_dir = proj_dirs.cache_dir();
        fs::create_dir_all(cache_dir)?;
        Ok(cache_dir.to_path_buf())
    } else {
        // Use current directory if no home dir is found.
        Ok(std::env::current_dir().unwrap().join(".opengate_cache"))
    }
}

pub fn get_models_dir() -> std::io::Result<PathBuf> {
    let sysconfig_dir = get_sysconfig_dir()?;
    let result = sysconfig_dir.join("models");
    if !result.exists() {
        info!("creating: {}", result.display());
    }
    fs::create_dir_all(&result)?;
    Ok(result)
}

pub fn get_audio_dir(audio_key: String) -> std::io::Result<PathBuf> {
    let sysconfig_dir = get_sysconfig_dir()?;
    let result = sysconfig_dir.join("audio").join(audio_key);
    if !result.exists() {
        info!("creating: {}", result.display());
    }
    fs::create_dir_all(&result)?;
    Ok(result)
}

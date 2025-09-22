use std::path::{Path, PathBuf};

fn expand_tilde(p: &Path) -> PathBuf {
    if let Some(s) = p.to_str()
        && s.starts_with("~")
    {
        return PathBuf::from(shellexpand::tilde(s).into_owned());
    }
    p.to_path_buf()
}

/// We want to resolve onnx models and audio files relatively to some base directory based on the
/// config parent dir, however, we might not even be passed a path - something like `audio_dir` or
/// `model_dir` might not be defined. In that case, we return `None` as well
pub fn resolve_relative(base_dir: &Path, maybe_path: &Option<PathBuf>) -> Option<PathBuf> {
    maybe_path.as_ref().map(|p| {
        let expanded = expand_tilde(p);
        if expanded.is_absolute() {
            // This was _not_ a relative directory.
            expanded
        } else {
            // Join it with, for our use cases, the parent directory of the yaml config file.
            base_dir.join(expanded)
        }
    })
}

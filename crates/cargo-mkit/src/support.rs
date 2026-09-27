use crate::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
pub(crate) fn cleanup(paths: &[PathBuf]) {
    for p in paths.iter().rev() {
        let _ = fs::remove_file(p);
    }
}
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("path has no parent")?;
    fs::create_dir_all(parent)?;
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp, bytes)?;
    fs::rename(&temp, path)?;
    Ok(())
}

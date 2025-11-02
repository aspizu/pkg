use std::fs::create_dir;
use std::path::PathBuf;

use eyre::Context;
use libmeow::meowzip::ensure_extension_is_mz;

pub fn extract(path: PathBuf, dir: Option<PathBuf>) -> eyre::Result<()> {
    ensure_extension_is_mz(&path)?;
    let dir = dir.unwrap_or_else(|| path.with_extension(""));
    create_dir(dir).context("Failed to create extraction directory")?;
    todo!()
}

use std::fs;

use anyhow::Context;
use fxhash::FxHashMap;
use tokio::process::Command;

use crate::{
    config::Config,
    manifest::Manifest,
};

pub type Index = FxHashMap<String, Manifest>;

pub fn _load_index(root: &str) -> anyhow::Result<Option<Index>> {
    if !fs::exists(format!("{}/var/lib/pkg/index.toml", root))? {
        return Ok(None);
    }
    let index_str = fs::read_to_string(format!("{}/var/lib/pkg/index.toml", root))
        .context("Failed to read index")?;
    let index: Index = toml::from_str(&index_str).context("Invalid index format")?;
    Ok(Some(index))
}

pub async fn update_index(root: &str, config: &Config) -> anyhow::Result<Index> {
    Command::new("/usr/bin/wget")
        .args([
            "-O",
            &format!("{}/var/lib/pkg/index.toml", root),
            &format!("{}/index.toml", config.index),
        ])
        .output()
        .await?
        .exit_ok()
        .context("Failed to update index")?;
    let index_str = fs::read_to_string(format!("{}/var/lib/pkg/index.toml", root))
        .context("Failed to read index")?;
    let index: Index = toml::from_str(&index_str).context("Invalid index format")?;
    Ok(index)
}

pub fn resolve_dependencies(index: &Index, packages: &mut Vec<String>, manifest: &Manifest) {
    for depname in &manifest.dependencies {
        if packages.contains(depname) {
            continue;
        };
        let depmanifest = &index[depname];
        resolve_dependencies(index, packages, depmanifest);
    }
    packages.push(manifest.name.clone());
}

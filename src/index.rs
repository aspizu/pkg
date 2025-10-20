use std::fs;

use anyhow::Context;
use fxhash::FxHashMap;
use tokio::process::Command;

use crate::{
    config::Config,
    manifest::Manifest,
};

pub type Index = FxHashMap<String, Manifest>;

pub fn _load_index() -> anyhow::Result<Option<Index>> {
    if !fs::exists("/var/lib/pkg/index.toml")? {
        return Ok(None);
    }
    let index_str =
        fs::read_to_string("/var/lib/pkg/index.toml").context("Failed to read index")?;
    let index: Index = toml::from_str(&index_str).context("Invalid index format")?;
    Ok(Some(index))
}

pub async fn update_index(config: &Config) -> anyhow::Result<Index> {
    Command::new("/usr/bin/wget")
        .args([
            "-O",
            "/var/lib/pkg/index.toml",
            &format!("{}/index.toml", config.index),
        ])
        .output()
        .await?
        .exit_ok()
        .context("Failed to update index")?;
    let index_str =
        fs::read_to_string("/var/lib/pkg/index.toml").context("Failed to read index")?;
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

use std::{
    env::current_dir,
    fs,
    path::PathBuf,
};

use anyhow::{
    Context,
    bail,
};
use tokio::process::Command;

use crate::manifest::Manifest;

pub async fn build(name: String) -> anyhow::Result<()> {
    let cwd = current_dir()?;
    let recipy_path = cwd.join("recipes").join(name);
    let manifest_path = recipy_path.join("manifest.toml");
    let manifest_src = fs::read_to_string(manifest_path).context("Failed to read manifest.toml")?;
    let manifest: Manifest =
        toml::from_str(&manifest_src).context("Failed to parse manifest.toml")?;
    fs::create_dir_all("/tmp/pkg/sources")?;
    Command::new("/usr/bin/wget")
        .args(["-c", &manifest.source])
        .current_dir("/tmp/pkg/sources")
        .status()
        .await?
        .exit_ok()
        .context("Failed to download source tarball")?;
    let Some((_, tarball_name)) = manifest.source.rsplit_once('/') else {
        bail!("Invalid source URL");
    };
    let Some(tarball_stem) = tarball_name
        .rsplit_once('.')
        .map(|(name, _)| name.strip_suffix(".tar").unwrap_or(name))
    else {
        bail!("Invalid source filename");
    };
    let tarball_path = PathBuf::from("/tmp/pkg/sources").join(tarball_stem);
    if tarball_path.try_exists()? {
        fs::remove_dir_all(&tarball_path)?;
    }
    let build_dir = PathBuf::from("/tmp/pkg/builds").join(manifest.fullname());
    if build_dir.try_exists()? {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(&build_dir)?;
    Command::new("/usr/bin/tar")
        .args(["-xf", tarball_name])
        .current_dir("/tmp/pkg/sources")
        .status()
        .await?
        .exit_ok()
        .context("Failed to extract source tarball")?;
    Command::new("/usr/bin/bash")
        .args(["-e", &format!("{}/build.sh", recipy_path.display())])
        .current_dir(tarball_path)
        .env("DESTDIR", &build_dir)
        .status()
        .await?
        .exit_ok()
        .context("Build script failed")?;
    let pkg_path =
        PathBuf::from("/tmp/pkg/builds").join(format!("{}.tar.zst", manifest.fullname()));
    Command::new("/usr/bin/tar")
        .args(["-acf", pkg_path.to_str().unwrap(), "."])
        .current_dir(build_dir)
        .status()
        .await?
        .exit_ok()
        .context("Failed to create package tarball")?;
    println!("Built {}", pkg_path.display());
    Ok(())
}

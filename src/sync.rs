use std::fs;

use anyhow::{
    Context,
    bail,
};
use tokio::process::Command;

use crate::{
    config::load_config,
    index::{
        resolve_dependencies,
        update_index,
    },
    manifest::load_manifest,
    package,
};

pub async fn sync() -> anyhow::Result<()> {
    let config = load_config()?;
    let index = update_index(&config).await?;
    let mut packages: Vec<String> = vec![];
    for package_name in &config.packages {
        let Some(manifest) = index.get(package_name) else {
            bail!("Package {} does not exist in the index", package_name);
        };
        resolve_dependencies(&index, &mut packages, manifest);
    }
    let mut to_upgrade: Vec<&str> = vec![];
    for package_name in &packages {
        let old_manifest_path = format!("/var/lib/pkg/{}/manifest.toml", &package_name);
        let old_manifest = if fs::exists(&old_manifest_path)? {
            Some(load_manifest(&old_manifest_path)?)
        } else {
            None
        };
        let new_manifest = &index[package_name];
        if let Some(old_manifest) = &old_manifest {
            if old_manifest == new_manifest {
                continue;
            }
        }
        to_upgrade.push(&package_name);
    }
    for package_name in &to_upgrade {
        let manifest = &index[*package_name];
        Command::new("/usr/bin/wget")
            .args([
                "-O",
                &format!(
                    "/tmp/pkg/tarballs/{}.tar.zst",
                    &format!("{}/{}.tar.zst", &config.index, manifest.fullname())
                ),
            ])
            .status()
            .await?
            .exit_ok()
            .context("Failed to download package tarball")?;
    }
    for package_name in to_upgrade {
        let manifest = &index[package_name];
        package::install(
            manifest,
            &format!("/tmp/pkg/tarballs/{}.tar.zst", manifest.fullname()),
        )
        .await?;
    }
    for entry in fs::read_dir("/var/lib/pkg")? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if ["index.toml", "lockfile.txt"].contains(&name) {
            continue;
        }
        if packages.iter().find(|p| *p == name).is_some() {
            continue;
        }
        package::uninstall(name).await?;
    }
    Ok(())
}

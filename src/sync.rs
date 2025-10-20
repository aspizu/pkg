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

pub async fn sync(root: Option<String>) -> anyhow::Result<()> {
    let root = root.unwrap_or_default();
    fs::create_dir_all(format!("{}/tmp/pkg/tarballs", &root))?;
    let config = load_config(&root)?;
    let index = update_index(&root, &config).await?;
    let mut packages: Vec<String> = vec![];
    for package_name in &config.packages {
        let Some(manifest) = index.get(package_name) else {
            bail!("Package {} does not exist in the index", package_name);
        };
        resolve_dependencies(&index, &mut packages, manifest);
    }
    let mut to_upgrade: Vec<&str> = vec![];
    for package_name in &packages {
        let old_manifest_path = format!("{}/var/lib/pkg/{}/manifest.toml", &root, &package_name);
        let old_manifest = if fs::exists(&old_manifest_path)? {
            Some(load_manifest(&old_manifest_path)?)
        } else {
            None
        };
        let new_manifest = &index[package_name];
        if let Some(old_manifest) = &old_manifest
            && old_manifest == new_manifest
        {
            continue;
        }
        to_upgrade.push(package_name);
    }
    for package_name in &to_upgrade {
        let manifest = &index[*package_name];
        Command::new("/usr/bin/wget")
            .args([
                "-C",
                &format!("{}/{}.tar.zst", &config.index, manifest.fullname()),
            ])
            .current_dir(&format!("{}/tmp/pkg/tarballs", &root))
            .status()
            .await?
            .exit_ok()
            .context("Failed to download package tarball")?;
    }
    for package_name in to_upgrade {
        let manifest = &index[package_name];
        package::install(
            &root,
            manifest,
            &format!("{}/tmp/pkg/tarballs/{}.tar.zst", &root, manifest.fullname()),
        )
        .await?;
    }
    for entry in fs::read_dir(format!("{}/var/lib/pkg", root))? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if ["index.toml", "lockfile.txt"].contains(&name) {
            continue;
        }
        if packages.iter().any(|needed| *needed == name) {
            continue;
        }
        package::uninstall(&root, name).await?;
    }
    Ok(())
}

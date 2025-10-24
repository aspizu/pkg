use std::{
    fs,
    fs::File,
    io::Read,
};

use eyre::{
    Context,
    bail,
};
use minisign_verify::Signature;
use tokio::process::Command;

use crate::{
    config::{
        load_config,
        load_keys,
    },
    index::{
        resolve_dependencies,
        update_index,
    },
    manifest::load_manifest,
    package,
};

async fn wget(url: &str, cwd: &str) -> eyre::Result<()> {
    Command::new("/usr/bin/wget")
        .args(["-nc", url])
        .current_dir(cwd)
        .status()
        .await?
        .exit_ok()?;
    Ok(())
}

pub async fn sync(root: Option<String>) -> eyre::Result<()> {
    let root = root.unwrap_or_default();
    fs::create_dir_all("/tmp/meow/meowzips")?;
    let config = load_config(&root)?;
    let keys = load_keys(&config)?;
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
        let old_manifest_path = format!(
            "{}/var/lib/meow/installed/{}/manifest.toml",
            &root, &package_name
        );
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
        wget(
            &format!("{}/{}.mz", &config.index, manifest.fullname()),
            "/tmp/meow/meowzips",
        )
        .await
        .context("Failed to download package meowzip")?;
        wget(
            &format!("{}/{}.mz.minisig", &config.index, manifest.fullname()),
            "/tmp/meow/meowzips",
        )
        .await
        .context("Failed to download package signature")?;
        let signature = Signature::from_file(&format!(
            "/tmp/meow/meowzips/{}.mz.minisig",
            manifest.fullname()
        ))
        .context("Failed to read package signature.")?;
        let mzpath = format!("/tmp/meow/meowzips/{}.mz", manifest.fullname());

        // Verify signature with any of the available keys
        let mut verified = false;
        for key in &keys {
            let mut file = File::open(&mzpath)?;
            let mut verifier = key.verify_stream(&signature)?;
            let mut buffer = [0u8; 8192]; // 8KB buffer

            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .context("Error reading package file")?;
                if bytes_read == 0 {
                    break; // End of file
                }
                verifier.update(&buffer[..bytes_read]);
            }

            // Try to verify with this key
            if verifier.finalize().is_ok() {
                verified = true;
                break;
            }
        }

        if !verified {
            bail!(
                "Signature verification failed for package {}",
                manifest.fullname()
            );
        }
    }
    for package_name in to_upgrade {
        let manifest = &index[package_name];
        package::install(
            &root,
            manifest,
            &format!("/tmp/meow/meowzips/{}.mz", manifest.fullname()),
        )?;
    }
    for entry in fs::read_dir(format!("{}/var/lib/meow/installed", root))? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if packages.iter().any(|needed| *needed == name) {
            continue;
        }
        package::uninstall(&root, name)?;
    }
    Ok(())
}

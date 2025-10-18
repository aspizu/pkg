use std::{
    fs::{
        self,
        File,
    },
    io::{
        BufRead,
        BufReader,
        BufWriter,
        Write,
    },
};

use anyhow::{
    Context,
    bail,
};
use tokio::process::Command;

use crate::manifest::{
    Manifest,
    load_manifest,
    save_manifest,
};

fn load_filelist(name: &str) -> anyhow::Result<Vec<String>> {
    let file = File::open(format!("/var/lib/pkg/{}/filelist.txt", name))
        .context("Failed to open filelist")?;
    let reader = BufReader::new(file);
    let mut lines = vec![];
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

async fn get_filelist(package: &str) -> anyhow::Result<Vec<String>> {
    let output = Command::new("/usr/bin/tar")
        .args(["-tf", package])
        .output()
        .await?
        .exit_ok()
        .context("Failed to get file list from package")?;
    let mut lines = vec![];
    for line in output.stdout.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

fn save_filelist(name: &str, filelist: &[String]) -> anyhow::Result<()> {
    let file = File::create(format!("/var/lib/pkg/{}/filelist.txt", name))
        .context("Failed to create filelist")?;
    let mut writer = BufWriter::new(file);
    for line in filelist {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

async fn unpack_package(path: &str) -> anyhow::Result<()> {
    Command::new("/usr/bin/tar")
        .args(["--overwrite", "-C", "/", "-xf", path])
        .status()
        .await?
        .exit_ok()
        .context("Failed to unpack package")?;
    Ok(())
}

fn uninstall_path(path: &str) -> anyhow::Result<()> {
    if !fs::exists(path)? {
        return Ok(());
    }
    let meta = fs::metadata(path)?;
    if meta.is_file() || meta.is_symlink() {
        fs::remove_file(path)?;
    } else if meta.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

pub async fn install(manifest: &Manifest, package: &str) -> anyhow::Result<()> {
    let filelist = get_filelist(package).await?;
    let path = format!("/var/lib/pkg/{}", &manifest.name);
    let old_data = if fs::exists(&path)? {
        let old_manifest =
            load_manifest(&format!("/var/lib/pkg/{}/manifest.toml", &manifest.name))?;
        let old_filelist = load_filelist(&manifest.name)?;
        Some((old_manifest, old_filelist))
    } else {
        None
    };
    unpack_package(package).await?;
    if let Some((old_manifest, old_filelist)) = old_data {
        for file in old_filelist {
            if filelist.contains(&file) {
                continue;
            }
            uninstall_path(&file)?;
        }
    }
    fs::create_dir_all(path)?;
    save_manifest(
        manifest,
        &format!("/var/lib/pkg/{}/manifest.toml", &manifest.name),
    )?;
    save_filelist(&manifest.name, &filelist)?;
    todo!()
}

pub async fn uninstall(name: &str) -> anyhow::Result<()> {
    let path = format!("/var/lib/pkg/{}", name);
    if !fs::exists(&path)? {
        bail!("Package {} is not installed", name);
    }
    let manifest_path = format!("/var/lib/pkg/{}/manifest.toml", name);
    let manifest = load_manifest(&manifest_path)?;
    let filelist = load_filelist(name)?;
    for file in filelist {
        uninstall_path(&file)?;
    }
    fs::remove_dir_all(path)?;
    Ok(())
}

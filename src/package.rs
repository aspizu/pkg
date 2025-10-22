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

use eyre::{
    Context,
    bail,
};
use tokio::process::Command;

use crate::manifest::{
    Manifest,
    load_manifest,
    save_manifest,
};

fn load_filelist(root: &str, name: &str) -> eyre::Result<Vec<String>> {
    let file = File::open(format!("{}/var/lib/pkg/{}/filelist.txt", root, name))
        .context("Failed to open filelist")?;
    let reader = BufReader::new(file);
    let mut lines = vec![];
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

async fn get_filelist(package: &str) -> eyre::Result<Vec<String>> {
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

fn save_filelist(root: &str, name: &str, filelist: &[String]) -> eyre::Result<()> {
    let file = File::create(format!("{}/var/lib/pkg/{}/filelist.txt", root, name))
        .context("Failed to create filelist")?;
    let mut writer = BufWriter::new(file);
    for line in filelist {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

async fn unpack_package(root: &str, path: &str) -> eyre::Result<()> {
    Command::new("/usr/bin/tar")
        .args(["--overwrite", "-C", &format!("{}/", root), "-xf", path])
        .status()
        .await?
        .exit_ok()
        .context("Failed to unpack package")?;
    Ok(())
}

fn uninstall_path(path: &str) -> eyre::Result<()> {
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

pub async fn install(root: &str, manifest: &Manifest, package: &str) -> eyre::Result<()> {
    let filelist = get_filelist(package).await?;
    let path = format!("{}/var/lib/pkg/{}", root, &manifest.name);
    let old_data = if fs::exists(&path)? {
        let old_manifest = load_manifest(&format!(
            "{}/var/lib/pkg/{}/manifest.toml",
            root, &manifest.name
        ))?;
        let old_filelist = load_filelist(root, &manifest.name)?;
        Some((old_manifest, old_filelist))
    } else {
        None
    };
    unpack_package(root, package).await?;
    if let Some((_old_manifest, old_filelist)) = old_data {
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
        &format!("{}/var/lib/pkg/{}/manifest.toml", root, &manifest.name),
    )?;
    save_filelist(root, &manifest.name, &filelist)?;
    Ok(())
}

pub async fn uninstall(root: &str, name: &str) -> eyre::Result<()> {
    let path = format!("{}/var/lib/pkg/{}", root, name);
    if !fs::exists(&path)? {
        bail!("Package {} is not installed", name);
    }
    let manifest_path = format!("{}/var/lib/pkg/{}/manifest.toml", root, name);
    let _manifest = load_manifest(&manifest_path)?;
    let filelist = load_filelist(root, name)?;
    for file in filelist {
        uninstall_path(&file)?;
    }
    fs::remove_dir_all(path)?;
    Ok(())
}

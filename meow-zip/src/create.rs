use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};

use eyre::{Context, bail};
use libmeow::meowzip::ensure_extension_is_mz;
use minisign::SecretKey;

pub fn create(
    path: PathBuf,
    name: String,
    version: String,
    release: u64,
    depends: String,
    packager: String,
    license: String,
) -> eyre::Result<()> {
    if release == 0 {
        bail!("Release number must be greater than 0");
    }
    ensure_extension_is_mz(&path)?;
    let mut file = File::create_new(&path).context("Failed to create meowzip file")?;
    file.write_all(b"MEOW")?;
    file.write_all(&(name.len() as u64).to_be_bytes())?;
    file.write_all(name.as_bytes())?;
    file.write_all(&(version.len() as u64).to_be_bytes())?;
    file.write_all(version.as_bytes())?;
    file.write_all(&release.to_be_bytes())?;
    file.write_all(&(packager.len() as u64).to_be_bytes())?;
    file.write_all(packager.as_bytes())?;
    file.write_all(&(license.len() as u64).to_be_bytes())?;
    file.write_all(license.as_bytes())?;
    file.write_all(&(depends.len() as u64).to_be_bytes())?;
    file.write_all(depends.as_bytes())?;
    write_hook(&mut file, "pre-install")?;
    write_hook(&mut file, "post-install")?;
    write_hook(&mut file, "pre-remove")?;
    write_hook(&mut file, "post-remove")?;
    let cwd = PathBuf::from(".");
    let mut filelist = vec![];
    get_filelist(cwd, &mut filelist)?;
    file.write_all(&(filelist.len() as u64).to_be_bytes())?;
    for path in &filelist {
        write_file_entry(&mut file, path)?;
    }
    file.write_all(b"ZSTD")?;
    let mut enc = zstd::Encoder::new(file, 0)?;
    for path in filelist {
        write_file_data(&mut enc, &path)?;
    }
    enc.finish()?;
    append_signature(&path)?;
    Ok(())
}

fn write_hook(out: &mut File, hook_name: &str) -> eyre::Result<()> {
    if !fs::exists(hook_name).context("Failed to open hook file")? {
        out.write_all(&0u64.to_be_bytes())?;
        return Ok(());
    }
    let meta = fs::metadata(hook_name)?;
    let mut file = File::open(hook_name)?;
    out.write_all(&(meta.len() as u64).to_be_bytes())?;
    io::copy(&mut file, out)?;
    Ok(())
}

fn open_minisign_secret_key() -> eyre::Result<SecretKey> {
    let Some(home_dir) = dirs::home_dir() else {
        bail!("Home directory not found");
    };
    let skpath = home_dir.join(".minisign/minisign.key");
    if !fs::exists(&skpath)? {
        bail!("minisign secret key does not exist. Generate it using `minisign -G`")
    }
    SecretKey::from_file(skpath, None).context("Failed to open minisign secret key")
}

fn append_signature(path: &Path) -> eyre::Result<()> {
    let sk = open_minisign_secret_key()?;
    let mut file = File::open(path).context("Failed to open meowzip file")?;
    let sig = minisign::sign(None, &sk, &mut file, None, None)?.to_bytes();
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .context("Failed to append meowzip file")?;
    file.write_all(&sig)?;
    file.write_all(&(sig.len() as u64).to_be_bytes())?;
    println!();
    Ok(())
}

fn get_filelist(dir: PathBuf, out: &mut Vec<PathBuf>) -> eyre::Result<()> {
    let mut entries = vec![];
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        entries.push(entry.path());
    }
    entries.sort();
    out.push(dir);
    for entry in entries {
        if [
            "./pre-install",
            "./post-install",
            "./pre-remove",
            "./post-remove",
        ]
        .contains(&entry.to_str().unwrap())
        {
            continue;
        }
        if entry.is_file() || entry.is_symlink() {
            out.push(entry);
        } else if entry.is_dir() {
            get_filelist(entry, out)?;
        }
    }
    Ok(())
}

fn write_file_entry(out: &mut File, path: &Path) -> eyre::Result<()> {
    let filepath = path.to_str().unwrap().strip_prefix('.').unwrap();
    let filepath = if filepath.is_empty() { "/" } else { filepath };
    let meta = fs::symlink_metadata(path)?;
    let size: u64 = if meta.is_dir() && !meta.is_symlink() {
        0
    } else {
        meta.st_size()
    };
    let checksum = libmeow::file_checksum(path)?;
    out.write_all(&(filepath.len() as u64).to_be_bytes())?;
    out.write_all(filepath.as_bytes())?;
    out.write_all(&size.to_be_bytes())?;
    out.write_all(&meta.st_mode().to_be_bytes())?;
    out.write_all(&meta.st_uid().to_be_bytes())?;
    out.write_all(&meta.st_gid().to_be_bytes())?;
    out.write_all(&checksum.to_be_bytes())?;
    Ok(())
}

fn write_file_data<T>(out: &mut T, path: &Path) -> eyre::Result<()>
where T: Write {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_symlink() {
        let link = fs::read_link(path)?;
        let link = link.to_str().unwrap().as_bytes();
        out.write_all(link)?;
        return Ok(());
    }
    if meta.is_dir() {
        return Ok(());
    }
    let mut file = File::open(path)?;
    io::copy(&mut file, out)?;
    Ok(())
}

use std::{
    fs::{
        self,
        File,
    },
    io::{
        self,
        BufRead,
        BufReader,
        BufWriter,
        Read,
        Write,
    },
    os::unix::{
        self,
    },
    path::{
        Path,
        PathBuf,
    },
    time::SystemTime,
};

use eyre::{
    Context,
    bail,
};
use file_mode::Mode;
use tokio::process::Command;

use crate::{
    manifest::{
        Manifest,
        load_manifest,
        save_manifest,
    },
    meowzip::{
        container::MeowZipReader,
        writer::hash_file,
    },
};

fn load_filelist(root: &str, name: &str) -> eyre::Result<Vec<String>> {
    let file = File::open(format!("{}/var/lib/meow/{}/filelist.txt", root, name))
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
    let file = File::create(format!("{}/var/lib/meow/{}/filelist.txt", root, name))
        .context("Failed to create filelist")?;
    let mut writer = BufWriter::new(file);
    for line in filelist {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

async fn unpack_package(root: &str, path: &str) -> eyre::Result<()> {
    Command::new("/usr/bin/tar")
        .args([
            "--overwrite",              // overwrite existing files when extracting
            "--no-overwrite-dir",       // preserve metadata of existing directories
            "--keep-directory-symlink", // preserve directory symlinks
            "-C",
            &format!("{}/", root),
            "-xf",
            path,
        ])
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
    let path = format!("{}/var/lib/meow/{}", root, &manifest.name);
    let old_data = if fs::exists(&path)? {
        let old_manifest = load_manifest(&format!(
            "{}/var/lib/meow/{}/manifest.toml",
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
        &format!("{}/var/lib/meow/{}/manifest.toml", root, &manifest.name),
    )?;
    save_filelist(root, &manifest.name, &filelist)?;
    Ok(())
}

pub async fn uninstall(root: &str, name: &str) -> eyre::Result<()> {
    let path = format!("{}/var/lib/meow/{}", root, name);
    if !fs::exists(&path)? {
        bail!("Package {} is not installed", name);
    }
    let manifest_path = format!("{}/var/lib/meow/{}/manifest.toml", root, name);
    let _manifest = load_manifest(&manifest_path)?;
    let filelist = load_filelist(root, name)?;
    for file in filelist {
        uninstall_path(&file)?;
    }
    fs::remove_dir_all(path)?;
    Ok(())
}

fn upgrade(mzpath: &str, root: &str) -> eyre::Result<()> {
    let file = File::open(mzpath)?;
    let (mut mzlist, mut mzdata) = MeowZipReader::new(file)?;
    if root != "" {
        for entry in &mut mzlist {
            let path = entry.path.to_str().unwrap();
            let joined = format!("{}{}", root, path);
            entry.path = PathBuf::from(joined);
        }
    }
    for entry in &mzlist {
        let mode = Mode::from(entry.mode);

        let prevmeta = fs::symlink_metadata(&entry.path).ok();

        let is_directory = mode
            .file_type()
            .is_some_and(|file_type| file_type.is_directory());
        let is_symlink = mode
            .file_type()
            .is_some_and(|file_type| file_type.is_symbolic_link());

        if is_symlink {
            if let Some(prevmeta) = prevmeta {
                // this pkg wants this path to be a symlink, but the system contains a non-symlink directory here
                // what? this should never happen, we delete the directory and its contents and replace it.
                if prevmeta.is_dir() && !prevmeta.is_symlink() {
                    std::fs::remove_dir_all(&entry.path)?;
                }
            }
            let mut link = String::new();
            mzdata.next_file(&mzlist).read_to_string(&mut link)?;
            // atomically create or overwrite existing symlink
            unix::fs::symlink(link, "/tmp/meow-transit")?;
            unix::fs::chown("/tmp/meow-transit", Some(entry.uid), Some(entry.gid))?;
            std::fs::rename("/tmp/meow-transit", &entry.path)?;
            // mode of the symlink file itself doesn't matter, should always be 777 ?
        } else {
            if is_directory {
                // directory already exists, it was probably created by this package
                // or some other package. we keep the existing permissions, and allow changes
                // by the sysadmin
                if prevmeta.is_some() {
                    continue;
                }
                fs::create_dir(&entry.path)?;
                unix::fs::chown(&entry.path, Some(entry.uid), Some(entry.gid))?;
                mode.apply_to_path(&entry.path)?;
                // directory entries are empty, their size value is nonsense
            } else {
                // the order in meow zip files are guaranteed to be sorted, such that
                // parent directories are created before their contents
                if let Some(prevmeta) = prevmeta {
                    let mtime = prevmeta.modified()?;
                    // sysadmin has customized this file
                    if mtime != SystemTime::UNIX_EPOCH {
                        let current_hash = hash_file(&entry.path)?;
                    }
                }
                let file = File::create(&entry.path)?;
                file.set_modified(SystemTime::UNIX_EPOCH)?;
                let mut writer = BufWriter::new(file);
                io::copy(&mut mzdata.next_file(&mzlist), &mut writer)?;
                unix::fs::chown(&entry.path, Some(entry.uid), Some(entry.gid))?;
                mode.apply_to_path(&entry.path)?;
            }
        }
    }
    Ok(())
}

use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::os::linux::fs::MetadataExt;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;

use atomic_file_install::{atomic_install, atomic_symlink_file};
use eyre::{Context, bail};
use file_mode::{FileType, Mode};
use libmeow::meowdb::FileRecord;
use libmeow::meowzip::{self, MeowZipEntry, MeowZipMetadata, ensure_extension_is_mz};
use libmeow::{columned, ensure_superuser, meowdb, path_chroot};
use redb::{ReadOnlyTable, ReadableDatabase};

use crate::remove::uninstall_path;

pub fn install(path: PathBuf, overwrite: bool, breakdeps: bool, root: PathBuf) -> eyre::Result<()> {
    ensure_superuser()?;
    ensure_extension_is_mz(&path)?;
    let mut mz = BufReader::new(File::open(&path).context("Failed to open package file")?);
    let pkgmeta = meowzip::read_metadata(&mut mz)?;
    let mut mz = zstd::Decoder::new(mz)?;
    let db = meowdb::open(&root)?;
    let read_txn = db.begin_read()?;
    let pkgs_table = read_txn.open_table(meowdb::PACKAGES)?;
    let files_table = read_txn.open_table(meowdb::FILES)?;

    let mut missing = vec![];
    for dependency in &pkgmeta.depends {
        if pkgs_table.get(dependency.as_str())?.is_none() {
            missing.push(dependency.as_str());
        }
    }
    if !missing.is_empty() && !breakdeps {
        println!("The following dependencies are missing:");
        columned::print(&missing);
        bail!(
            "Cannot install package due to missing dependencies, use `--breakdeps` to install anyway"
        );
    }

    let oldpkgmeta = pkgs_table.get(&*pkgmeta.name)?.map(|row| MeowZipMetadata::from(row.value()));
    if oldpkgmeta.as_ref().is_some() && !overwrite {
        bail!("Package '{}' is already installed, use `--overwrite` to reinstall", pkgmeta.name);
    }

    let mut path_contexts = vec![];
    for entry in &pkgmeta.filelist {
        path_contexts.push(get_path_context(entry, &files_table, &root)?);
        let ctx = path_contexts.last().unwrap();
        check_conflicts(&pkgmeta, entry, ctx)?;
    }

    if &root == "/" {
        run_hook(
            &pkgmeta.name,
            &pkgmeta.pre_install,
            "pre-install",
            oldpkgmeta.as_ref().map(|m| m.version.as_str()).unwrap_or_default(),
            &pkgmeta.version,
        )?;
    }

    for (entry, ctx) in pkgmeta.filelist.iter().zip(&path_contexts) {
        if !ctx.filetype.is_directory() {
            continue;
        };
        if let Some(oldmeta) = &ctx.oldmeta {
            if oldmeta.is_symlink() || oldmeta.is_file() {
                fs::remove_file(&entry.filepath)?;
            }
            if oldmeta.is_dir() && !oldmeta.is_symlink() {
                continue;
            }
        }
        let dest = path_chroot(&entry.filepath, &root);
        fs::create_dir_all(&dest)?;
        unix::fs::lchown(&dest, Some(entry.uid), Some(entry.gid))?;
        Mode::from(entry.mode).set_mode_path(dest)?;
    }

    for (entry, ctx) in pkgmeta.filelist.iter().zip(path_contexts) {
        if ctx.filetype.is_directory() {
            continue;
        }
        let mut dest = path_chroot(&entry.filepath, &root);
        let mut entrydata = mz.by_ref().take(entry.size);
        match ctx.filetype {
            FileType::SymbolicLink => {
                if let Some(oldmeta) = ctx.oldmeta {
                    if oldmeta.is_dir() && !oldmeta.is_symlink() {
                        fs::remove_dir_all(&dest)?;
                    }
                }
                let mut targetpath = String::new();
                entrydata.read_to_string(&mut targetpath)?;
                let targetpath = PathBuf::from(targetpath);
                atomic_symlink_file(&targetpath, &dest)?;
            }
            FileType::RegularFile => {
                if let Some(oldmeta) = &ctx.oldmeta {
                    if oldmeta.is_dir() && !oldmeta.is_symlink() {
                        fs::remove_dir_all(&dest)?;
                    }
                }
                let org = ctx.oldrecord.map(|oldrecord| oldrecord.checksum).unwrap_or(0);
                let cur = if ctx.oldmeta.is_some() { libmeow::file_checksum(&dest)? } else { 0 };
                let new = entry.checksum;
                let mut discard = false;

                // X-X-X
                if org == cur && cur == new {
                    discard = true;
                }

                // X-X-Y
                if org == cur && cur != new {
                    // Upgrade the old file.
                    discard = false;
                }

                // X-Y-X
                if org == new && cur != new {
                    discard = true;
                }

                // X-Y-Y
                if org != cur && cur == new {
                    discard = true;
                }

                // X-Y-Z
                if org != cur && cur != new && org != new {
                    let basedest = dest;
                    dest = basedest.with_added_extension("pacnew");
                    let mut i = 2;
                    while fs::exists(&dest)? {
                        dest = basedest
                            .with_added_extension("pacnew")
                            .with_added_extension(i.to_string());
                        i += 1;
                    }
                    println!(
                        "warning: `{}` installed as `{}`",
                        &entry.filepath.display(),
                        dest.display()
                    );
                }

                if discard {
                    io::copy(&mut entrydata, &mut io::sink())?;
                } else {
                    // atomic_install will copy to parent of dest if not on same filesystem
                    let tmpdest = PathBuf::from("/tmp/meow-pkg-tempfile");
                    let mut newfile = File::create(&tmpdest)?;
                    io::copy(&mut entrydata, &mut newfile)?;
                    atomic_install(&tmpdest, &dest)?;
                }
            }
            _ => bail!("invalid file type in meowzip {}", entry.filepath.display()),
        }
        unix::fs::lchown(&dest, Some(entry.uid), Some(entry.gid))?;
        Mode::from(entry.mode).set_mode_path(&dest)?;
    }

    let write_txn = db.begin_write()?;
    {
        let mut pkgs_table = write_txn.open_table(meowdb::PACKAGES)?;
        let mut files_table = write_txn.open_table(meowdb::FILES)?;

        if let Some(old_pkgmeta) = &oldpkgmeta {
            for entry in old_pkgmeta.filelist.iter().rev() {
                if pkgmeta.filelist.iter().any(|e| e.filepath == entry.filepath) {
                    continue;
                }
                uninstall_path(&root, &entry.filepath, &mut files_table)?;
            }
        }

        for entry in &pkgmeta.filelist {
            let record = FileRecord::from(entry).with_package(pkgmeta.name.clone());
            let row = bincode::encode_to_vec(&record, bincode::config::standard())?;
            files_table.insert(&entry.filepath.to_str().unwrap(), &*row)?;
        }

        let metadata_bytes = bincode::encode_to_vec(&pkgmeta, bincode::config::standard())?;
        pkgs_table.insert(pkgmeta.name.as_str(), metadata_bytes.as_slice())?;
    }
    write_txn.commit()?;

    if &root == "/" {
        run_hook(
            &pkgmeta.name,
            &pkgmeta.post_remove,
            "post-install",
            oldpkgmeta.as_ref().map(|m| m.version.as_str()).unwrap_or_default(),
            &pkgmeta.version,
        )?;
    }

    Ok(())
}

pub fn run_hook(
    package_name: &str,
    hook: &[u8],
    hook_name: &str,
    arg0: &str,
    arg1: &str,
) -> eyre::Result<()> {
    let hook = str::from_utf8(hook).unwrap();
    Command::new("/usr/bin/bash")
        .args(["-c", hook, arg0, arg1])
        .status()?
        .exit_ok()
        .with_context(|| format!("The package `{}`'s `{}` hook failed", package_name, hook_name))?;
    Ok(())
}

struct PathContext {
    filetype: FileType,
    oldrecord: Option<FileRecord>,
    oldmeta: Option<fs::Metadata>,
}

fn get_path_context(
    entry: &MeowZipEntry,
    files_table: &ReadOnlyTable<&str, &[u8]>,
    root: &Path,
) -> eyre::Result<PathContext> {
    let filepath = path_chroot(&entry.filepath, root);
    Ok(PathContext {
        filetype: Mode::from(entry.mode).file_type().unwrap(),
        oldrecord: files_table
            .get(entry.filepath.to_str().unwrap())?
            .map(|row| FileRecord::from(row.value())),
        oldmeta: if fs::exists(&filepath)? { Some(fs::symlink_metadata(filepath)?) } else { None },
    })
}

fn check_conflicts(
    pkgmeta: &MeowZipMetadata,
    entry: &MeowZipEntry,
    ctx: &PathContext,
) -> eyre::Result<()> {
    let alreadyowned =
        ctx.oldrecord.as_ref().is_some_and(|oldrecord| oldrecord.package != pkgmeta.name);
    if (ctx.filetype.is_symbolic_link() || ctx.filetype.is_regular_file()) && alreadyowned {
        bail!(
            "conflict: `{}` is already owned by package `{}`",
            entry.filepath.display(),
            ctx.oldrecord.as_ref().unwrap().package
        );
    }
    let Some(oldmeta) = &ctx.oldmeta else {
        return Ok(());
    };
    let upgradable = ctx.oldrecord.as_ref().is_some_and(|oldrecord| {
        let oldrecordmode = Mode::from(oldrecord.mode);
        let oldmetamode = Mode::from(oldmeta.st_mode());
        let oldrecordfiletype = oldrecordmode.file_type().unwrap();
        let oldmetafiletype = oldmetamode.file_type().unwrap();
        oldrecordfiletype == oldmetafiletype
    });
    if upgradable {
        return Ok(());
    }
    match &ctx.filetype {
        FileType::SymbolicLink => {
            if !oldmeta.is_symlink() {
                bail!("conflict: `{}` is not a symlink", &entry.filepath.display());
            }
        }
        FileType::RegularFile => {
            if !oldmeta.is_file() {
                bail!("conflict: `{}` is not a regular file", &entry.filepath.display());
            }
        }
        FileType::Directory => {
            if oldmeta.is_symlink() || !oldmeta.is_dir() {
                bail!("conflict: `{}` is not a directory", &entry.filepath.display());
            }
        }
        _ => bail!("invalid file type in meowzip {}", entry.filepath.display()),
    }
    Ok(())
}

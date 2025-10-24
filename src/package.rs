use std::{
    fs::{
        self,
        File,
        OpenOptions,
    },
    io::{
        self,
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

use file_mode::Mode;

use crate::{
    manifest::{
        Manifest,
        save_manifest,
    },
    meowzip::{
        container::{
            MZlistWriter,
            MeowZipReader,
        },
        reader::Entry,
        writer::hash_file,
    },
};

fn install_meowzip(name: &str, mzpath: &str, root: &str) -> eyre::Result<()> {
    let mzlistpath = format!("{}/var/lib/meow/installed/{}/mzlist", root, name);
    let oldmzlist = if fs::exists(&mzlistpath)? {
        let file = File::open(&mzlistpath)?;
        let (oldmzlist, _) = MeowZipReader::new(file)?;
        oldmzlist
    } else {
        vec![]
    };
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
            if let Some(prevmeta) = &prevmeta {
                // this pkg wants this path to be a symlink, but the system contains a non-symlink directory here
                // what? this should never happen, we delete the directory and its contents and replace it.
                if prevmeta.is_dir() && !prevmeta.is_symlink() {
                    std::fs::remove_dir_all(&entry.path)?;
                }
            }
            let mut link = String::new();
            mzdata.next_file(&mzlist).read_to_string(&mut link)?;
            if prevmeta.is_some() {
                fs::remove_file(&entry.path)?;
            }
            unix::fs::symlink(link, &entry.path)?;
            unix::fs::lchown(&entry.path, Some(entry.uid), Some(entry.gid))?;
            // mode of the symlink file itself doesn't matter, should always be 777 ?
        } else {
            if is_directory {
                mzdata.skip_file(&mzlist)?;
                // directory already exists, it was probably created by this package
                // or some other package. we keep the existing permissions, and allow changes
                // by the sysadmin
                if prevmeta.is_some() {
                    continue;
                }
                fs::create_dir(&entry.path)?;
                unix::fs::lchown(&entry.path, Some(entry.uid), Some(entry.gid))?;
                mode.apply_to_path(&entry.path)?;
                // directory entries are empty, their size value is nonsense
            } else {
                // the order in meow zip files are guaranteed to be sorted, such that
                // parent directories are created before their contents
                if let Some(prevmeta) = prevmeta {
                    let mtime = prevmeta.modified()?;
                    // sysadmin has customized this file
                    if mtime != SystemTime::UNIX_EPOCH {
                        let org = oldmzlist
                            .iter()
                            .find(|x| x.path == entry.path)
                            .map(|x| x.hash)
                            .unwrap_or(0);
                        let cur = hash_file(&entry.path)?;
                        let new = entry.hash;
                        // X-X-X
                        if org == cur && org == new {
                            // no operation
                        }
                        // X-X-Y
                        else if org == cur && org != new {
                            // overwrite existing
                        }
                        // X-Y-X
                        else if org == new && org != cur {
                            // discard the file
                            mzdata.skip_file(&mzlist)?;
                            // skip overwriting
                            continue;
                        }
                        // X-Y-Y
                        else if org != cur && cur == new {
                            // no operation
                        }
                        // X-Y-Z
                        else {
                            // save upgrade as a copy
                            let mut new_dest = entry.path.with_added_extension("meow-upgrade");
                            let mut i = 2;
                            while new_dest.exists() {
                                new_dest = entry
                                    .path
                                    .with_added_extension("meow-upgrade")
                                    .with_added_extension(i.to_string());
                                i += 1;
                            }
                            install_file(&mzlist, &mut mzdata, entry, mode, &new_dest)?;
                            // log the upgrade path
                            let mut file = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&format!("{}/var/lib/meow/upgradable-files.txt", root))?;
                            file.write_all(new_dest.to_str().unwrap().as_bytes())?;
                            file.write_all(b"\n")?;
                            continue;
                        }
                    }
                }
                install_file(&mzlist, &mut mzdata, entry, mode, &entry.path)?;
            }
        }
    }
    install_mzlist(&mzlist, &mzlistpath)?;
    Ok(())
}

fn install_mzlist(mzlist: &[Entry], mzlistpath: &str) -> eyre::Result<()> {
    let file = File::create(mzlistpath)?;
    let writer = BufWriter::new(file);
    let mzlistwriter = MZlistWriter::new(writer, mzlist)?;
    mzlistwriter.finish()?;
    Ok(())
}

fn install_file<T>(
    mzlist: &[Entry],
    mzdata: &mut MeowZipReader<T>,
    entry: &Entry,
    mode: Mode,
    dest: &Path,
) -> io::Result<()>
where
    T: Read,
{
    let file = File::create(dest)?;
    file.set_modified(SystemTime::UNIX_EPOCH)?;
    let mut writer = BufWriter::new(file);
    io::copy(&mut mzdata.next_file(&mzlist), &mut writer)?;
    unix::fs::lchown(dest, Some(entry.uid), Some(entry.gid))?;
    mode.set_mode_path(dest)?;
    Ok(())
}

pub fn install(root: &str, manifest: &Manifest, mzpath: &str) -> eyre::Result<()> {
    fs::create_dir_all(&format!(
        "{}/var/lib/meow/installed/{}",
        root, &manifest.name
    ))?;
    install_meowzip(&manifest.name, mzpath, root)?;
    save_manifest(
        manifest,
        &format!(
            "{}/var/lib/meow/installed/{}/manifest.toml",
            root, &manifest.name
        ),
    )?;
    Ok(())
}

pub fn uninstall(root: &str, name: &str) -> eyre::Result<()> {
    let mzpath = &format!("{}/var/lib/meow/installed/{}/mzlist", root, name);
    let file = File::open(mzpath)?;
    let (mut mzlist, _) = MeowZipReader::new(file)?;
    // in reverse order, meow zip files are removed first, then their parent dirs are
    mzlist.reverse();
    for entry in mzlist {
        uninstall_file(&entry.path)?;
    }
    Ok(())
}

fn uninstall_file(path: &Path) -> io::Result<()> {
    if !fs::exists(path)? {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_symlink() || metadata.is_file() {
        fs::remove_file(path)?;
    } else if metadata.is_dir() {
        match fs::remove_dir(path) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::DirectoryNotEmpty => (),
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

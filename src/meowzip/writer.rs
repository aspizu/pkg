use std::{
    fs::{
        self,
        OpenOptions,
    },
    hash::Hasher,
    io::{
        self,
        BufReader,
        Write,
    },
    os::{
        linux::fs::MetadataExt,
        unix::fs::OpenOptionsExt,
    },
    path::{
        Path,
        PathBuf,
    },
};

use xxhash_rust::xxh3;

use crate::meowzip::reader::Entry;

fn get_filelist(dir: PathBuf, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries = vec![];
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        entries.push(entry.path());
    }
    entries.sort();
    out.push(dir);
    for entry in entries {
        if entry.is_dir() && !entry.is_symlink() {
            get_filelist(entry, out)?;
        } else {
            out.push(entry);
        }
    }
    Ok(())
}

pub fn write_archive<T>(out: &mut T) -> io::Result<()>
where T: Write {
    let cwd = PathBuf::from(".");
    let mut filelist = vec![];
    get_filelist(cwd, &mut filelist)?;
    out.write_all(&filelist.len().to_le_bytes())?;
    for entry in &filelist {
        write_entry(&mut *out, entry)?;
    }
    for entry in &filelist {
        if entry.is_dir() && !entry.is_symlink() {
            continue;
        }
        write_file(&mut *out, entry)?;
    }
    Ok(())
}

pub fn hash_file(path: &Path) -> io::Result<u64> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = xxh3::Xxh3Default::new();
    io::copy(&mut reader, &mut hasher)?;
    let hash = hasher.finish();
    Ok(hash)
}

fn write_entry<T>(out: &mut T, path: &Path) -> io::Result<()>
where T: Write {
    let filename = path.to_str().unwrap().strip_prefix('.').unwrap();
    let filename = if filename == "" { "/" } else { filename };
    let meta = fs::symlink_metadata(path)?;
    let is_dir = meta.is_dir();
    let is_symlink = meta.is_symlink();
    out.write_all(&filename.len().to_le_bytes())?;
    out.write_all(filename.as_bytes())?;
    out.write_all(
        &(if is_dir && !is_symlink {
            0
        } else {
            meta.st_size()
        })
        .to_le_bytes(),
    )?;
    out.write_all(&meta.st_mode().to_le_bytes())?;
    out.write_all(&meta.st_uid().to_le_bytes())?;
    out.write_all(&meta.st_gid().to_le_bytes())?;
    out.write_all(
        &(if is_symlink || is_dir {
            0
        } else {
            hash_file(path)?
        })
        .to_le_bytes(),
    )?;
    Ok(())
}

fn write_file<T>(out: &mut T, path: &Path) -> io::Result<()>
where T: Write {
    if path.is_symlink() {
        let file = path.read_link()?;
        out.write_all(file.to_str().unwrap().as_bytes())?;
    } else {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?;
        let mut reader = BufReader::new(file);
        io::copy(&mut reader, out)?;
    }
    Ok(())
}

pub fn write_mzlist<T>(out: &mut T, mzlist: &[Entry]) -> io::Result<()>
where T: Write {
    out.write_all(&mzlist.len().to_le_bytes())?;
    for entry in mzlist {
        let path = entry.path.to_str().unwrap().as_bytes();
        out.write_all(&path.len().to_le_bytes())?;
        out.write_all(path)?;
        out.write_all(&entry.size.to_le_bytes())?;
        out.write_all(&entry.mode.to_le_bytes())?;
        out.write_all(&entry.uid.to_le_bytes())?;
        out.write_all(&entry.gid.to_le_bytes())?;
        out.write_all(&entry.hash.to_le_bytes())?;
    }
    Ok(())
}

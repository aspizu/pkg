use std::io::Read;
use std::path::{Path, PathBuf};

use bincode::{Decode, Encode};
use eyre::bail;

#[derive(Encode, Decode)]
pub struct MeowZipMetadata {
    pub name: String,
    pub version: String,
    pub release: u64,
    pub depends: Vec<String>,
    pub packager: String,
    pub license: String,
    pub pre_install: Vec<u8>,
    pub post_install: Vec<u8>,
    pub pre_remove: Vec<u8>,
    pub post_remove: Vec<u8>,
    pub filelist: Vec<MeowZipEntry>,
}

#[derive(Encode, Decode)]
pub struct MeowZipEntry {
    pub filepath: PathBuf,
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub checksum: u64,
}

pub fn read_metadata<T>(file: &mut T) -> eyre::Result<MeowZipMetadata>
where T: Read {
    let mut buf_u32 = [0u8; 4];
    let mut buf_u64 = [0u8; 8];
    file.read_exact(&mut buf_u32)?;
    if &buf_u32 != b"MEOW" {
        bail!("Not a valid meowzip file");
    }
    file.read_exact(&mut buf_u64)?;
    let name_len = u64::from_be_bytes(buf_u64) as usize;
    let mut name_buf = vec![0u8; name_len];
    file.read_exact(&mut name_buf)?;
    let name = String::from_utf8(name_buf)?;
    file.read_exact(&mut buf_u64)?;
    let version_len = u64::from_be_bytes(buf_u64) as usize;
    let mut version_buf = vec![0u8; version_len];
    file.read_exact(&mut version_buf)?;
    let version = String::from_utf8(version_buf)?;
    file.read_exact(&mut buf_u64)?;
    let release = u64::from_be_bytes(buf_u64);
    file.read_exact(&mut buf_u64)?;
    let packager_len = u64::from_be_bytes(buf_u64) as usize;
    let mut packager_buf = vec![0u8; packager_len];
    file.read_exact(&mut packager_buf)?;
    let packager = String::from_utf8(packager_buf)?;
    file.read_exact(&mut buf_u64)?;
    let license_len = u64::from_be_bytes(buf_u64) as usize;
    let mut license_buf = vec![0u8; license_len];
    file.read_exact(&mut license_buf)?;
    let license = String::from_utf8(license_buf)?;
    file.read_exact(&mut buf_u64)?;
    let depends_len = u64::from_be_bytes(buf_u64) as usize;
    let mut depends_buf = vec![0u8; depends_len];
    file.read_exact(&mut depends_buf)?;
    let depends = String::from_utf8(depends_buf)?;
    file.read_exact(&mut buf_u64)?;
    let pre_install_len = u64::from_be_bytes(buf_u64) as usize;
    let mut pre_install = vec![0u8; pre_install_len];
    file.read_exact(&mut pre_install)?;
    file.read_exact(&mut buf_u64)?;
    let post_install_len = u64::from_be_bytes(buf_u64) as usize;
    let mut post_install = vec![0u8; post_install_len];
    file.read_exact(&mut post_install)?;
    file.read_exact(&mut buf_u64)?;
    let pre_remove_len = u64::from_be_bytes(buf_u64) as usize;
    let mut pre_remove = vec![0u8; pre_remove_len];
    file.read_exact(&mut pre_remove)?;
    file.read_exact(&mut buf_u64)?;
    let post_remove_len = u64::from_be_bytes(buf_u64) as usize;
    let mut post_remove = vec![0u8; post_remove_len];
    file.read_exact(&mut post_remove)?;
    file.read_exact(&mut buf_u64)?;
    let file_count = u64::from_be_bytes(buf_u64);
    let mut filelist = vec![];
    for _ in 0..file_count {
        file.read_exact(&mut buf_u64)?;
        let filepath_len = u64::from_be_bytes(buf_u64) as usize;
        let mut filepath_buf = vec![0u8; filepath_len];
        file.read_exact(&mut filepath_buf)?;
        let filepath = String::from_utf8(filepath_buf)?;
        let filepath = PathBuf::from(filepath);
        file.read_exact(&mut buf_u64)?;
        let size = u64::from_be_bytes(buf_u64);
        file.read_exact(&mut buf_u32)?;
        let mode = u32::from_be_bytes(buf_u32);
        file.read_exact(&mut buf_u32)?;
        let uid = u32::from_be_bytes(buf_u32);
        file.read_exact(&mut buf_u32)?;
        let gid = u32::from_be_bytes(buf_u32);
        file.read_exact(&mut buf_u64)?;
        let checksum = u64::from_be_bytes(buf_u64);

        filelist.push(MeowZipEntry {
            filepath,
            size,
            mode,
            uid,
            gid,
            checksum,
        });
    }
    file.read_exact(&mut buf_u32)?;
    if &buf_u32 != b"ZSTD" {
        bail!(format!(
            "I don't know how to decompress the compression format {}",
            str::from_utf8(&buf_u32).unwrap_or("UNKNOWN")
        ));
    }
    Ok(MeowZipMetadata {
        name,
        version,
        release,
        depends: depends.split(',').map(|s| s.trim().to_string()).collect(),
        packager,
        license,
        pre_install,
        post_install,
        pre_remove,
        post_remove,
        filelist,
    })
}

pub fn ensure_extension_is_mz(path: &Path) -> eyre::Result<()> {
    if path
        .extension()
        .is_none_or(|ext| ext.to_str().unwrap() != "mz")
    {
        bail!("File extension must be `.mz`");
    }
    Ok(())
}

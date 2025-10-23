use std::{
    io::Read,
    path::PathBuf,
};

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub size: usize,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub hash: u64,
}

pub fn get_filelist<T>(reader: &mut T) -> eyre::Result<Vec<Entry>>
where T: Read {
    let mut filelist = vec![];
    let mut u32_buf = [0; 4];
    let mut u64_buf = [0; 8];

    reader.read_exact(&mut u64_buf)?;
    let filelist_len = usize::from_le_bytes(u64_buf);

    filelist.reserve_exact(filelist_len);

    for _ in 0..filelist_len {
        reader.read_exact(&mut u64_buf)?;
        let filename_len = usize::from_le_bytes(u64_buf);

        let mut filename = vec![0; filename_len];
        reader.read_exact(&mut filename)?;
        let path = PathBuf::from(str::from_utf8(&filename)?);

        reader.read_exact(&mut u64_buf)?;
        let size = usize::from_le_bytes(u64_buf);

        reader.read_exact(&mut u32_buf)?;
        let mode = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let uid = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let gid = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u64_buf)?;
        let hash = u64::from_le_bytes(u64_buf);

        filelist.push(Entry {
            path,
            size,
            mode,
            uid,
            gid,
            hash,
        })
    }
    Ok(filelist)
}

pub fn split_archive(filelist: &[Entry]) -> eyre::Result<usize> {
    let mut length = 0;

    // filelist count
    length += 8;

    for entry in filelist {
        // filename length
        length += 8;

        // filename
        length += entry.path.to_str().unwrap().len();

        // size
        length += 8;

        // mode
        length += 4;

        // uid
        length += 4;

        // gid
        length += 4;

        // hash
        length += 8;
    }

    Ok(length)
}

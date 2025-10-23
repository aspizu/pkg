use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
};

use eyre::Context;
use file_mode::Mode;
use human_bytes::human_bytes;

use crate::meowzip::container::{
    MeowZipReader,
    MeowZipWriter,
};

pub async fn zip(path: PathBuf, list: bool) -> eyre::Result<()> {
    if list {
        let file = File::open(path).context("Failed to open input file.")?;
        let (mzlist, _) = MeowZipReader::new(file)?;
        for entry in &mzlist {
            let mode = Mode::from(entry.mode);
            assert!(mode.mode() == entry.mode);
            println!(
                "{} {}:{} {:>8} {:<64} {:X}",
                mode.to_string(),
                entry.uid,
                entry.gid,
                human_bytes(entry.size as f64),
                entry.path.display(),
                entry.hash,
            );
        }
        let total_size: usize = mzlist.iter().map(|x| x.size).sum();
        println!("Total size {}", human_bytes(total_size as f64))
    } else {
        let file = File::create(path).context("Failed to open output file.")?;
        let writer = BufWriter::new(file);
        let meowzip = MeowZipWriter::new(writer)?;
        meowzip.finish()?;
    }
    Ok(())
}

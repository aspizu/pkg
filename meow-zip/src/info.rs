use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use eyre::Context;
use humansize::{DECIMAL, format_size};
use libmeow::meowzip::{self, ensure_extension_is_mz};

pub fn info(path: PathBuf) -> eyre::Result<()> {
    ensure_extension_is_mz(&path)?;
    let file = File::open(&path).context("Failed to open meowzip file")?;
    let mut reader = BufReader::new(file);
    let metadata = meowzip::read_metadata(&mut reader)?;
    let total_size: u64 = metadata.filelist.iter().map(|entry| entry.size).sum();
    println!("Name:        {}", metadata.name);
    println!("Version:     {}", metadata.version);
    println!("Release:     {}", metadata.release);
    println!("Depends:     {}", metadata.depends.join(", "));
    println!("Packager:    {}", metadata.packager);
    println!("License:     {}", metadata.license);
    println!("Total Files: {}", metadata.filelist.len());
    println!("Total Size:  {}", format_size(total_size, DECIMAL));
    Ok(())
}

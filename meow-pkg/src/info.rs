use std::path::PathBuf;

use eyre::bail;
use humansize::format_size;
use libmeow::meowdb;
use libmeow::meowzip::MeowZipMetadata;
use redb::ReadableDatabase;

pub fn info(root: PathBuf, package: String) -> eyre::Result<()> {
    let db = meowdb::open(&root)?;
    let read_txn = db.begin_read()?;
    let pkgs_table = read_txn.open_table(meowdb::PACKAGES)?;
    let Some(row) = pkgs_table.get(&*package)? else {
        bail!("Package `{}` is not installed", package);
    };
    let metadata = MeowZipMetadata::from(row.value());
    let total_size: u64 = metadata.filelist.iter().map(|entry| entry.size).sum();
    println!("Name:        {}", metadata.name);
    println!("Version:     {}", metadata.version);
    println!("Release:     {}", metadata.release);
    println!("Depends:     {}", metadata.depends.join(", "));
    println!("Packager:    {}", metadata.packager);
    println!("License:     {}", metadata.license);
    println!("Total Files: {}", metadata.filelist.len());
    println!("Total Size:  {}", format_size(total_size, humansize::DECIMAL));
    todo!()
}

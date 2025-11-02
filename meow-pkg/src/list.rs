use std::path::PathBuf;

use libmeow::meowdb;
use redb::{ReadableDatabase, ReadableTable};

pub fn list(root: PathBuf) -> eyre::Result<()> {
    let db = meowdb::open(&root)?;
    let read_txn = db.begin_read()?;
    let pkgs_table = read_txn.open_table(meowdb::PACKAGES)?;
    for result in pkgs_table.iter()? {
        let (_, value) = result?;
        let metadata = libmeow::meowzip::MeowZipMetadata::from(value.value());
        println!("{}-{}-{}.mz", metadata.name, metadata.version, metadata.release);
    }
    Ok(())
}

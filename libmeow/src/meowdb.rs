use std::path::Path;

use eyre::Context;
use redb::{Database, TableDefinition};

use crate::meowzip::{MeowZipEntry, MeowZipMetadata};

const DB_PATH: &str = "/var/lib/meow.db";

pub fn open(root: &Path) -> eyre::Result<redb::Database> {
    Database::create(root.join(DB_PATH)).context("Failed to open or create the database")
}

pub const PACKAGES: TableDefinition<&str, &[u8]> = TableDefinition::new("PKGS");

/// Should not store directories
pub const FILES: TableDefinition<&str, &[u8]> = TableDefinition::new("FILES");

#[derive(bincode::Decode, bincode::Encode)]
pub struct FileRecord {
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub checksum: u64,
    /// Name of the package that owns this file or symlink
    pub package: String,
}

impl From<&MeowZipEntry> for FileRecord {
    fn from(value: &MeowZipEntry) -> Self {
        FileRecord {
            size: value.size,
            mode: value.mode,
            uid: value.uid,
            gid: value.gid,
            checksum: value.checksum,
            package: String::new(),
        }
    }
}

impl FileRecord {
    pub fn with_package(mut self, package: String) -> Self {
        self.package = package;
        self
    }
}

impl From<&[u8]> for FileRecord {
    fn from(value: &[u8]) -> Self {
        bincode::decode_from_slice(value, bincode::config::standard()).unwrap().0
    }
}

impl From<&[u8]> for MeowZipMetadata {
    fn from(value: &[u8]) -> Self {
        bincode::decode_from_slice(value, bincode::config::standard()).unwrap().0
    }
}

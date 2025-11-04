pub mod columned;
pub mod meowdb;
pub mod meowzip;

use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{self, BufReader};
use std::path::Path;

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use xxhash_rust::xxh3::Xxh3Default;

pub const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

pub fn file_checksum(path: &Path) -> eyre::Result<u64> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_symlink() || meta.is_dir() {
        return Ok(0);
    }
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Xxh3Default::new();
    io::copy(&mut reader, &mut hasher)?;
    let checksum = hasher.finish();
    Ok(checksum)
}

pub fn ensure_superuser() -> eyre::Result<()> {
    if !nix::unistd::getuid().is_root() {
        eyre::bail!("This operation requires superuser privileges");
    }
    Ok(())
}

pub const MEOW: &'static str = r#"  ,-.       _,---._ __  / \
 /  )    .-'       `./ /   \
(  (   ,'            `/    /|
 \  `-"             \'\   / |
  `.              ,  \ \ /  |
   /`.          ,'-`----Y   |
  (            ;        |   '
  |  ,-.    ,-'         |  /
  |  | (   |        hjw | /
  )  |  \  `.___________|/
  `--'   `--'"#;

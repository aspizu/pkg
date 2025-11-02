use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::install::install;
use crate::remove::remove;

#[derive(Parser)]
#[command(about = "Package manager for meowOS")]
#[command(styles=libmeow::CLAP_STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// The root directory (default: /)
    root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    Install {
        /// Path to meowzip package file to install
        package: PathBuf,
        /// Force reinstall if package is already installed
        #[arg(short, long)]
        force: bool,
    },
    Remove {
        /// Name of package to uninstall
        package: String,
    },
}

pub fn run() -> eyre::Result<()> {
    let args = Cli::parse();
    let root = args.root.unwrap_or_default();
    match args.command {
        Command::Install { package, force } => install(package, force, root),
        Command::Remove { package } => remove(package, root),
    }
}

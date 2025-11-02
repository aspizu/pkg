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
    match Cli::parse().command {
        Command::Install { package, force } => install(package, force),
        Command::Remove { package } => remove(package),
    }
}

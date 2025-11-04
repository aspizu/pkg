use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::info::info;
use crate::install::install;
use crate::list::list;
use crate::remove::remove;

#[derive(Parser)]
#[command(about = format!("{}{}", libmeow::MEOW, "Package manager for meowOS"))]
#[command(styles=libmeow::CLAP_STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// The root directory (default: /)
    #[arg(long)]
    root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    Install {
        /// Path to meowzip package file to install
        package: PathBuf,
        /// Force reinstall if package is already installed
        #[arg(long)]
        overwrite: bool,
        /// Break dependencies
        #[arg(long)]
        breakdeps: bool,
    },
    Remove {
        /// Name of package to uninstall
        package: String,
        /// Break dependencies
        #[arg(long)]
        breakdeps: bool,
    },
    /// List installed packages
    List,
    /// Show information about an installed package
    Info {
        /// Name of package to query
        package: String,
    },
}

pub fn run() -> eyre::Result<()> {
    let args = Cli::parse();
    let root = args.root.unwrap_or_default();
    match args.command {
        Command::Install { package, overwrite, breakdeps } => {
            install(package, overwrite, breakdeps, root)
        }
        Command::Remove { package, breakdeps } => remove(package, breakdeps, root),
        Command::List => list(root),
        Command::Info { package } => info(root, package),
    }
}

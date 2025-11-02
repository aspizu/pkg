use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::create::create;
use crate::extract::extract;
use crate::info::info;
use crate::list::list;
use crate::verify::verify;

#[derive(Parser)]
#[command(about = "Archive file format for meowOS packages")]
#[command(styles=libmeow::CLAP_STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(alias = "c")]
    /// Add all files in the current directory to a new meowzip file
    Create {
        /// Path to the meowzip file to create
        file: PathBuf,
        /// Package name
        #[arg(short, long)]
        name: String,
        /// Package version
        #[arg(short, long)]
        version: String,
        /// Package release number
        #[arg(short, long)]
        release: Option<u64>,
        /// Packager name and email
        #[arg(short, long)]
        packager: String,
        /// Package license SPDX identifier
        #[arg(short, long)]
        license: String,
        /// Dependencies comma separated
        #[arg(short, long)]
        depends: Option<String>,
    },
    #[command(alias = "x")]
    /// Extract all files from a meowzip file
    Extract {
        /// Path to the meowzip file to extract
        file: PathBuf,
        /// Directory to extract files to (defaults to new directory named after the meowzip file)
        dir: Option<PathBuf>,
    },
    #[command(alias = "l")]
    /// List the contents of a meowzip file
    List {
        /// Path to the meowzip file to list
        file: PathBuf,
    },
    #[command(alias = "i")]
    /// Show metadata
    Info {
        /// Path to the meowzip file to show metadata
        file: PathBuf,
    },
    /// Verify the signature of a meowzip file
    Verify {
        /// Path to the meowzip file to verify
        file: PathBuf,
        /// Public key value to use for verification
        #[arg(short, long)]
        publickey: String,
    },
}

pub fn run() -> eyre::Result<()> {
    match Cli::parse().command {
        Command::Create {
            file,
            name,
            version,
            release,
            depends,
            packager,
            license,
        } => create(
            file,
            name,
            version,
            release.unwrap_or(1),
            depends.unwrap_or_default(),
            packager,
            license,
        ),
        Command::Extract { file, dir } => extract(file, dir),
        Command::List { file } => list(file),
        Command::Info { file } => info(file),
        Command::Verify { file, publickey } => verify(file, publickey),
    }
}

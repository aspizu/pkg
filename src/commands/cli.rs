use std::path::PathBuf;

use clap::{
    CommandFactory,
    Parser,
};
use clap_derive::Subcommand;

use crate::commands::{
    reconfigure::reconfigure,
    sync::sync,
    zip::zip,
};

#[derive(Debug, Parser)]
#[command(version=env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The root directory of the system. (default: /)
    #[arg(short, long)]
    root: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Sync packages installed on this system to the index.
    #[command()]
    Sync,
    /// Manually reconfigure a package.
    #[command()]
    Reconfigure { package: String },
    /// Perform meow zip operations. Default operation is to zip current directory.
    Zip {
        /// Path to a meow zip file.
        file: PathBuf,
        /// Read and list contents of meow zip, instead of creating a new file.
        #[arg(short, long)]
        list: bool,
    },
    /// Generate completions for a shell.
    #[command()]
    Completions {
        /// The shell to generate the completions for.
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

pub async fn cli() -> eyre::Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        Command::Sync => {
            sync(args.root).await?;
        }
        Command::Reconfigure { package } => {
            reconfigure(&package)?;
        }
        Command::Zip { file, list } => {
            zip(file, list).await?;
        }
    }
    Ok(())
}

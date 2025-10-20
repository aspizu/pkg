use clap::{
    CommandFactory,
    Parser,
};
use clap_derive::Subcommand;

use crate::sync::sync;

#[derive(Debug, Parser)]
#[command(version=env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sync packages installed on this system to the index.
    #[command()]
    Sync,
    /// Generate completions for a shell.
    #[command()]
    Completions {
        /// The shell to generate the completions for.
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

pub async fn cli() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        Command::Sync => {
            sync().await?;
        }
    }
    Ok(())
}

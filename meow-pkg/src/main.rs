#![feature(exit_status_error)]

mod cli;
mod install;
mod remove;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    cli::run()
}

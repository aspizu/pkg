#![feature(exit_status_error)]

mod cli;
mod info;
mod install;
mod list;
mod remove;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    cli::run()
}

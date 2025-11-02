#![feature(seek_stream_len)]

mod cli;
mod create;
mod extract;
mod info;
mod list;
mod verify;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    cli::run()
}

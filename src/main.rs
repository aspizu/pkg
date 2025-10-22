#![feature(exit_status_error)]
mod cli;
mod config;
mod index;
mod manifest;
mod package;
mod sync;
use std::time::Instant;

use colored::{
    Color,
    Colorize,
};

use crate::cli::cli;
#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    let begin = Instant::now();
    std::panic::set_hook(Box::new(|info| {
        eprintln!(
            "{}\n\n{}",
            info,
            "You probably need to re-install your machine now."
                .red()
                .bold()
        );
    }));
    let result = cli().await;
    eprintln!(
        "{} in {:?}",
        "Finished"
            .color(if result.is_ok() {
                Color::Green
            } else {
                Color::Red
            })
            .bold(),
        begin.elapsed()
    );
    result
}

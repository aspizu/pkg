#![feature(exit_status_error)]

mod build;
mod cli;
mod config;
mod index;
mod manifest;
mod package;
mod sync;

use std::{
    process::ExitCode,
    time::Instant,
};

use colored::{
    Color,
    Colorize,
};

use crate::cli::cli;

#[tokio::main]
async fn main() -> ExitCode {
    pretty_env_logger::init();
    let begin = Instant::now();
    std::panic::set_hook(Box::new(|info| {
        eprintln!(
            "{info}\n{}\nopen an issue at {}",
            "neptune is cooked 💀".red().bold(),
            "https://github.com/aspizu/neptune/issues".cyan()
        );
    }));
    let result = cli().await;
    if let Err(error) = &result {
        eprintln!("{}{} {}", "error".bold().red(), ":".bold(), error);
    }
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
    if result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use eyre::Context;
use file_mode::{FileType, Mode, User};
use humansize::{DECIMAL, format_size};
use libmeow::meowzip::{self, ensure_extension_is_mz};
use owo_colors::OwoColorize;

pub fn list(path: PathBuf) -> eyre::Result<()> {
    ensure_extension_is_mz(&path)?;
    let file = File::open(path).context("Failed to open meowzip file")?;
    let mut reader = BufReader::new(file);
    let metadata = meowzip::read_metadata(&mut reader)?;
    let mut stack = vec![];
    let mut prefix = "";
    for entry in &metadata.filelist {
        let mode = Mode::from(entry.mode);
        let is_executable = mode.user_protection(User::Owner).is_execute_set()
            || mode.user_protection(User::Group).is_execute_set()
            || mode.user_protection(User::Other).is_execute_set();
        print!("{} ", mode);
        let filepath = entry.filepath.to_str().unwrap();
        let filename = loop {
            if let Some(filename) = filepath.strip_prefix(prefix) {
                break filename.strip_prefix("/").unwrap_or(filename);
            }
            prefix = stack.pop().unwrap();
        };
        let filetype = mode.file_type().unwrap();
        for _ in 0..stack.len() {
            print!("    ");
        }
        match filetype {
            FileType::SymbolicLink => print!("{}", filename.cyan()),
            FileType::Directory => print!("{}", filename.blue()),
            _ if is_executable => print!("{}", filename.green()),
            _ => print!("{}", filename),
        }
        if filetype.is_directory() {
            stack.push(prefix);
            prefix = filepath;
        }

        println!();
    }
    let total_size: u64 = metadata.filelist.iter().map(|entry| entry.size).sum();
    println!("Total {}", format_size(total_size, DECIMAL));
    Ok(())
}

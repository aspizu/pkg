#!/usr/bin/env bash
set -ex
cat << EOF | podman run --rm -i rust:trixie /usr/bin/bash
set -ex
mkdir /tmp/foo
ln -sf /tmp/foo /tmp/foo2
chown -h 69:420 /tmp/foo2
mkdir -p src
cat << BRUH > Cargo.toml
[package]
name = "test"
version = "1.0.0"
edition = "2024"
BRUH
cat << BRUH > src/main.rs
use std::io::Read;

fn main() -> std::io::Result<()> {
    let mut data = "Hello, World!".as_bytes(); // convert &str to &[u8]
    let mut reader = &mut data;
    let mut nbytes = reader.by_ref().take(3); // only read first 3 bytes
    let mut s = String::new();
    nbytes.read_to_string(&mut s)?; // pass mutable String
    println!("{}", s);
    Ok(())
}

BRUH
cargo run
ls -lha /tmp
stat /tmp/foo2
EOF

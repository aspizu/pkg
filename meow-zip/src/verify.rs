use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use eyre::Context;
use minisign::{PublicKey, PublicKeyBox, SignatureBox};

pub fn verify(path: PathBuf, publickey: String) -> eyre::Result<()> {
    let pk_box = PublicKeyBox::from(publickey);
    let pk = PublicKey::from_box(pk_box)?;
    let mut file = File::open(&path).context("Failed to open meowzip file")?;
    let len = file.stream_len()?;
    let mut u64_buf = [0u8; 8];
    file.seek(SeekFrom::End(-8))?;
    file.read_exact(&mut u64_buf)?;
    let signature_length = u64::from_be_bytes(u64_buf) as usize;
    file.seek(SeekFrom::End(-8 - signature_length as i64))?;
    let mut signature = vec![0; signature_length];
    file.read_exact(&mut signature)?;
    let signature = String::from_utf8(signature)?;
    file.seek(SeekFrom::Start(0))?;
    let file = file.by_ref().take(len - 8 - signature_length as u64);
    let signature_box = SignatureBox::from_string(&signature)?;
    minisign::verify(&pk, &signature_box, file, true, false, false)?;
    Ok(())
}

use std::io::{
    BufReader,
    Read,
    Take,
    Write,
};

use eyre::{
    Context,
    bail,
};

use crate::meowzip::{
    reader::{
        self,
        Entry,
    },
    writer,
};

pub struct MeowZipReader<T>
where T: Read
{
    inner: zstd::Decoder<'static, BufReader<T>>,
    i: usize,
}

impl<T> MeowZipReader<T>
where T: Read
{
    pub fn new(mut inner: T) -> eyre::Result<(Vec<Entry>, Self)> {
        let mut u32_buf = [0; 4];
        inner.read_exact(&mut u32_buf)?;

        if &u32_buf != b"MEOW" {
            bail!("Not a valid meow zip file.");
        }

        let mut decoder = zstd::Decoder::new(inner)?;

        let filelist = reader::get_filelist(&mut decoder).context("Not a valid meow zip file.")?;

        Ok((
            filelist,
            Self {
                inner: decoder,
                i: 0,
            },
        ))
    }

    pub fn next_file(
        &mut self,
        filelist: &[Entry],
    ) -> Take<&mut zstd::Decoder<'static, BufReader<T>>> {
        let reader = self.inner.by_ref().take(filelist[self.i].size as u64);
        self.i += 1;
        reader
    }
}

pub struct MeowZipWriter<T>
where T: Write
{
    pub inner: zstd::Encoder<'static, T>,
}

impl<T> MeowZipWriter<T>
where T: Write
{
    pub fn new(mut inner: T) -> eyre::Result<Self> {
        inner.write_all(b"MEOW")?;
        let mut encoder = zstd::Encoder::new(inner, 0)?;
        writer::write_archive(&mut encoder)?;
        Ok(Self { inner: encoder })
    }

    pub fn finish(self) -> eyre::Result<()> {
        self.inner.finish()?;
        Ok(())
    }
}

pub struct MZlistWriter<T>
where T: Write
{
    pub inner: zstd::Encoder<'static, T>,
}

impl<T> MZlistWriter<T>
where T: Write
{
    pub fn new(mut inner: T, mzlist: &[Entry]) -> eyre::Result<Self> {
        inner.write_all(b"MEOW")?;
        let mut encoder = zstd::Encoder::new(inner, 0)?;
        writer::write_mzlist(&mut encoder, mzlist)?;
        Ok(Self { inner: encoder })
    }

    pub fn finish(self) -> eyre::Result<()> {
        self.inner.finish()?;
        Ok(())
    }
}

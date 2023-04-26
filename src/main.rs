#![allow(clippy::print_literal)]
use std::{io::{BufReader, Read, Seek, SeekFrom}, path::PathBuf};

use clap::Parser;
use cpio::newc::Reader;
use sha2::{Digest, Sha256};

const BUF_SIZE: usize = 16384;
const BUF_SIZE_U64: u64 = 16384;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

#[derive(clap::Parser)]
#[clap(name = "cpio-dump")]
struct Cli {
    file: PathBuf,
    offset: Option<u64>,
}

fn main() {
    let args = Cli::parse();
    let mut handle = std::fs::File::open(args.file).unwrap();
    handle.seek(SeekFrom::End(0)).unwrap();
    let size = handle.stream_position().unwrap();

    if let Some(offset) = args.offset {
        handle.seek(SeekFrom::Start(offset)).unwrap();
    }

    let mut handle = BufReader::with_capacity(BUF_SIZE, handle);
    let mut hasher = Sha256::new();
    let mut read_buffer = [0; BUF_SIZE];

    println!(
        "{ino}\t{mode}\t{uid}\t{gid}\t{nlink}\t{mtime}\t{file_size}\t{dev_major}\t{dev_minor}\t{rdev_major}\t{rdev_minor}\t{is_trailer}\t{hash}\t{name}",
        ino="ino",
        mode="mode",
        uid="uid",
        gid="gid",
        nlink="nlink",
        mtime="mtime",
        file_size="bytes",
        dev_major="devmaj",
        dev_minor="devmin",
        rdev_major="rdevmaj",
        rdev_minor="rdevmin",
        is_trailer="trailer",
        hash="hash",
        name="name"
    );

    loop {
        // GNU cpio seems to pad the end of the file with some number
        // of 0s, which the reader is confused by. We don't want to prune
        // the null bytes, since they may be part of a file. However, we
        // can safely chomp them away before reading a cpio since the header
        // is a 070701.
        //
        // However, let's not bother doing that unless we're in the last few
        // kb where we're more likely to see padding.
        let start_position = handle.stream_position().unwrap();
        let remaining_bytes = size - start_position;
        if remaining_bytes <= BUF_SIZE_U64 {
            let skipped = skip_nulls(&mut handle).unwrap();
            if skipped > 0 {
                println!(
                    "{ino}\t{mode}\t{uid}\t{gid}\t{nlink}\t{mtime}\t{file_size}\t{dev_major}\t{dev_minor}\t{rdev_major}\t{rdev_minor}\t{is_trailer}\t{hash}\t{name}",
                    ino="-",
                    mode="-",
                    uid="-",
                    gid="-",
                    nlink="-",
                    mtime="-",
                    file_size=skipped,
                    dev_major="-",
                    dev_minor="-",
                    rdev_major="-",
                    rdev_minor="-",
                    is_trailer="-",
                    hash="-",
                    name="skipped null bytes",
                );
            }
        }

        let new_position = handle.stream_position().unwrap();
        if new_position >= size {
            break;
        }

        let mut entry = Reader::new(&mut handle).unwrap();
        let hash = hash(&mut hasher, &mut read_buffer, &mut entry).unwrap();
        let meta = entry.entry();
        println!(
            "{ino}\t{mode}\t{uid}\t{gid}\t{nlink}\t{mtime}\t{file_size}\t{dev_major}\t{dev_minor}\t{rdev_major}\t{rdev_minor}\t{is_trailer}\t{hash}\t{name}",
            ino=meta.ino(),
            mode=meta.mode(),
            uid=meta.uid(),
            gid=meta.gid(),
            nlink=meta.nlink(),
            mtime=meta.mtime(),
            file_size=meta.file_size(),
            dev_major=meta.dev_major(),
            dev_minor=meta.dev_minor(),
            rdev_major=meta.rdev_major(),
            rdev_minor=meta.rdev_minor(),
            is_trailer=if meta.is_trailer() { "true" } else { "false"},
            hash=hash,
            name=meta.name(),
        );

        entry.finish().unwrap();
    }
}

fn skip_nulls<T>(mut handle: T) -> Result<u64, std::io::Error>
where
    T: ReadSeek,
{
    let start_position = handle.stream_position()?;
    let mut silly_buffer = [0; 1];
    let nullread: &[u8; 1] = &[0x00];
    loop {
        match handle.read(&mut silly_buffer) {
            Ok(0) => {
                break;
            }
            Ok(1) => {
                if &silly_buffer != nullread {
                    handle.seek(SeekFrom::Current(-1))?;
                    break;
                }
            }

            e => {
                e?;
            }
        }
    }

    let new_position = handle.stream_position()?;
    Ok(new_position - start_position)
}

fn hash<T>(hasher: &mut Sha256, buffer: &mut [u8], handle: &mut T) -> Result<String, std::io::Error>
where
    T: std::io::Read,
{
    hasher.reset();
    loop {
        match handle.read(buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                hasher.update(&buffer[..n]);
            }
            e => {
                e.unwrap();
            }
        }
    }
    let result = hasher.finalize_reset();
    Ok(base64::encode(result))
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use sha2::{Digest, Sha256};

    use super::{hash, skip_nulls, BUF_SIZE};

    #[test]
    fn test_hash() {
        let mut hasher = Sha256::new();
        let mut buffer = [0; BUF_SIZE];

        assert_eq!(
            hash(&mut hasher, &mut buffer, &mut std::io::empty()).unwrap(),
            "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
        );

        assert_eq!(
            hash(&mut hasher, &mut buffer, &mut Cursor::new(b"hello")).unwrap(),
            "LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ="
        );
    }

    #[test]
    fn skip_nulls_eof() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![]);

        assert_eq!(skip_nulls(&mut data).unwrap(), 0);
        assert_eq!(data.position(), 0);
    }

    #[test]
    fn skip_nulls_none_eof() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![0x01]);

        assert_eq!(skip_nulls(&mut data).unwrap(), 0);
        assert_eq!(data.position(), 0);
    }

    #[test]
    fn skip_nulls_none_then_null() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![0x01, 0x00]);

        assert_eq!(skip_nulls(&mut data).unwrap(), 0);
        assert_eq!(data.position(), 0);
    }

    #[test]
    fn skip_nulls_null_then_data() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![0x00, 0x01]);

        assert_eq!(skip_nulls(&mut data).unwrap(), 1);
        assert_eq!(data.position(), 1);
        assert_eq!(data.bytes().next().unwrap().unwrap(), 0x01);
    }

    #[test]
    fn skip_nulls_many_null_then_data() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ]);

        assert_eq!(skip_nulls(&mut data).unwrap(), 10);
        assert_eq!(data.position(), 10);
        assert_eq!(data.bytes().next().unwrap().unwrap(), 0x01);
    }

    #[test]
    fn skip_nulls_seeked() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![0x01, 0x00, 0x01]);

        data.seek(SeekFrom::Start(1)).unwrap();

        assert_eq!(skip_nulls(&mut data).unwrap(), 1);
        assert_eq!(data.position(), 2);
        assert_eq!(data.bytes().next().unwrap().unwrap(), 0x01);
    }
}

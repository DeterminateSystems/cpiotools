use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

use cpio::newc::Reader;
use sha2::{Digest, Sha256};

const BUF_SIZE: usize = 16384;
const BUF_SIZE_U64: u64 = 16384;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

fn main() {
    let cpio = std::env::args().nth(1).unwrap();
    let handle = std::fs::File::open(cpio).unwrap();

    let size = handle.metadata().unwrap().len();

    let mut handle = BufReader::with_capacity(BUF_SIZE, handle);
    let mut hasher = Sha256::new();
    let mut read_buffer = [0; BUF_SIZE];

    println!("ino\tmode\tuid\tgid\tnlink\tmtime\tbytes\tdevmaj\tdevmin\trdevmaj\trdevmin\ttrailer\thash\tname");

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
                println!("Skipped {} null bytes", skipped);
            }
        }

        let new_position = handle.stream_position().unwrap();
        if new_position >= size {
            break;
        }
        let mut entry = Reader::new(&mut handle).unwrap();

        loop {
            match entry.read(&mut read_buffer) {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    hasher.update(&read_buffer[..n]);
                }
                e => {
                    e.unwrap();
                }
            }
        }
        let result = hasher.finalize_reset();

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
            hash=base64::encode(result),
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

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use super::skip_nulls;

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

use std::io::BufReader;

use base64;
use cpio::newc::Reader;
use sha2::{Digest, Sha256};

use std::convert::{TryInto, TryFrom};
use std::io::{Read, Seek, SeekFrom};

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
            loop {
                match handle.read(&mut read_buffer) {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        if !read_buffer[..n].iter().all(|e| e == &0x00) {
                            // roll back n bytes so we can nibble them away like an dummy
                            handle
                                .seek(SeekFrom::Current((0 - i32::try_from(n).unwrap()).into()))
                                .unwrap();

                            let mut silly_buffer = [0; 1];
                            let nullread: &[u8; 1] = &[0x00];
                            loop {
                                match handle.read(&mut silly_buffer) {
                                    Ok(0) => {
                                        break;
                                    }
                                    Ok(1) => {
                                        if &silly_buffer != nullread {
                                            handle.seek(SeekFrom::Current(-1)).unwrap();
                                            break;
                                        }
                                    }

                                    e => {
                                        e.unwrap();
                                    }
                                }
                            }
                            break;
                        }
                    }
                    e => {
                        e.unwrap();
                    }
                }
            }
        }

        let new_position = handle.stream_position().unwrap();
        if new_position != start_position {
            println!("Skipped {} null bytes", new_position - start_position);
        }

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

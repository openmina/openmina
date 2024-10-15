use std::io::{BufRead, Cursor, Seek, SeekFrom};

use super::util::{get_names, Leb128};

//#[derive(Debug)]
#[allow(dead_code)]
pub struct Names(Vec<String>);

impl Names {
    #[coverage(off)]
    pub fn new() -> Self {
        let names_buf = unsafe { get_names() }.to_vec();
        let mut cursor = Cursor::new(&names_buf);
        let mut output = Vec::new();
        let mut pos = cursor.position();
        let names_buf_len = names_buf.len() as u64;

        while pos < names_buf_len {
            let uncompressed_len = u64::read_leb128(&mut cursor);
            let compressed_len = u64::read_leb128(&mut cursor);

            pos = cursor.position();
            let start = pos as usize;

            if compressed_len == 0 {
                let end = start + uncompressed_len as usize;
                Self::read_names(&names_buf[start..end], &mut output);
                pos += uncompressed_len;
            } else {
                let end = start + compressed_len as usize;
                let decompressed_buf =
                    Self::decompress(&names_buf[start..end], uncompressed_len as usize);
                Self::read_names(decompressed_buf.as_slice(), &mut output);
                pos += compressed_len;
            }

            cursor.seek(SeekFrom::Start(pos)).unwrap();
        }

        Self(output)
    }

    #[coverage(off)]
    fn read_names(names: &[u8], output: &mut Vec<String>) {
        let mut names = Cursor::new(names);

        loop {
            let mut name = Vec::new();

            if names.read_until(0x01, &mut name).unwrap() == 0 {
                return;
            }

            if *name.last().unwrap() == 0x01 {
                name.pop();
            }

            output.push(String::from_utf8(name).unwrap());
        }
    }

    #[coverage(off)]
    fn decompress(compressed_buf: &[u8], uncompressed_len: usize) -> Vec<u8> {
        let mut decompressed_buf = vec![0; uncompressed_len];
        flate2::Decompress::new(true)
            .decompress(
                compressed_buf,
                &mut decompressed_buf,
                flate2::FlushDecompress::Finish,
            )
            .unwrap();
        decompressed_buf
    }
}

use super::util::{cursor_align, read_int, Leb128};
use std::io::{Cursor, Read, Seek, SeekFrom};

//#[derive(Debug)]
#[allow(dead_code)]
pub struct Header {
    zero: u32,
    encoded_filenames_len: u32,
    zero2: u32,
    version: u32,
}

impl Header {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        let header = Self {
            zero: read_int(cursor),
            encoded_filenames_len: read_int(cursor),
            zero2: read_int(cursor),
            version: read_int(cursor),
        };

        assert!(header.zero == 0 && header.zero2 == 0);
        header
    }
}

//#[derive(Debug)]
pub struct Filenames(pub Vec<String>);

impl Filenames {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        let num_filenames = u64::read_leb128(cursor);
        let uncompressed_len = usize::read_leb128(cursor);
        let compressed_len = usize::read_leb128(cursor);
        let pos = cursor.position() as usize;

        let mut filenames = Vec::new();
        let mut decompressed_buf = vec![0; uncompressed_len];
        let buf = if compressed_len != 0 {
            flate2::Decompress::new(true)
                .decompress(
                    &cursor.get_ref()[pos..pos + compressed_len],
                    &mut decompressed_buf[..],
                    flate2::FlushDecompress::Finish,
                )
                .unwrap();
            decompressed_buf
        } else {
            cursor.get_ref()[pos..pos + uncompressed_len].to_vec()
        };

        let mut cursor = Cursor::new(&buf);

        while (cursor.position() as usize) < uncompressed_len {
            filenames.push(Self::read_filename(&mut cursor));
        }

        assert!(filenames.len() == num_filenames as usize);
        Self(filenames)
    }

    #[coverage(off)]
    fn read_filename(cursor: &mut Cursor<&Vec<u8>>) -> String {
        let string_len = usize::read_leb128(cursor);
        let mut output = vec![0; string_len];
        cursor.read_exact(output.as_mut_slice()).unwrap();
        String::from_utf8(output).unwrap()
    }
}

//#[derive(Debug)]
#[allow(dead_code)]
pub struct CovMap {
    header: Header,
    pub encoded_data_hash: u64,
    pub filenames: Filenames,
}

impl CovMap {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        let header = Header::read(cursor);
        let pos = cursor.position() as usize;
        let end = pos + header.encoded_filenames_len as usize;
        let digest = md5::compute(&cursor.get_ref()[pos..end]).0;
        let encoded_data_hash: u64 = read_int(&mut Cursor::new(digest));
        let filenames = Filenames::read(cursor);
        cursor.seek(SeekFrom::Start(end as u64)).unwrap();
        cursor_align::<u64>(cursor);

        Self {
            header,
            encoded_data_hash,
            filenames,
        }
    }
}

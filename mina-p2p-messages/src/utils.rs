use std::io::Read;

use binprot::{
    byteorder::{LittleEndian, ReadBytesExt},
    BinProtRead,
};

/// Decodes an integer from `bin_prot` encoded bytes provided by the given reader.
pub fn decode_int<T, R>(r: &mut R) -> Result<T, binprot::Error>
where
    T: BinProtRead,
    R: Read,
{
    T::binprot_read(r)
}

/// Decodes a [String] from `bin_prot` encoded bytes provided by the given reader.
pub fn decode_string<R>(r: &mut R) -> Result<String, binprot::Error>
where
    R: Read,
{
    String::binprot_read(r)
}

/// Decodes an integer from the slice containing `bin_prot` encoded bytes.
/// Returns the resulting integer value and the number of bytes read from the
/// reader.
pub fn decode_int_from_slice<T>(slice: &[u8]) -> Result<(T, usize), binprot::Error>
where
    T: BinProtRead,
{
    let mut ptr = slice;
    Ok((decode_int(&mut ptr)?, slice.len() - ptr.len()))
}

/// Decodes a [String] from the slice containing `bin_prot` encoded bytes.
/// Returns the resulting value and the number of bytes read from the reader.
pub fn decode_string_from_slice(slice: &[u8]) -> Result<(String, usize), binprot::Error> {
    let mut ptr = slice;
    Ok((decode_string(&mut ptr)?, slice.len() - ptr.len()))
}

/// Returns an OCaml-like string view from the slice containing `bin_prot`
/// encoded bytes.
pub fn decode_bstr_from_slice(slice: &[u8]) -> Result<&[u8], binprot::Error> {
    let mut ptr = slice;
    let len = binprot::Nat0::binprot_read(&mut ptr)?.0 as usize;
    Ok(&ptr[..len])
}

/// Reads size of the next stream frame, specified by an 8-byte integer encoded as little-endian.
pub fn decode_size<R>(r: &mut R) -> Result<usize, binprot::Error>
where
    R: Read,
{
    let len = r.read_u64::<LittleEndian>()?;
    len.try_into()
        .map_err(|_| binprot::Error::CustomError("integer conversion".into()))
}

/// Returns a slice of bytes of lenght specified by first 8 bytes in little endian.
pub fn get_sized_slice(mut slice: &[u8]) -> Result<&[u8], binprot::Error> {
    let len = (&mut slice).read_u64::<LittleEndian>()? as usize;
    Ok(&slice[..len])
}

pub trait FromBinProtStream: BinProtRead + Sized {
    /// Decodes bytes from reader of byte stream into the specified type `T`. This
    /// function assumes that the data is prepended with 8-bytes little endian
    /// integer specirying the size.
    ///
    /// TODO: Even if not the whole portion of the stream is
    /// read to decode to `T`, reader is set to the end of the current stream
    /// portion, as specified by its size.
    fn read_from_stream<R>(r: &mut R) -> Result<Self, binprot::Error>
    where
        R: Read,
    {
        use std::io;
        let len = r.read_u64::<LittleEndian>()?;
        let mut r = r.take(len);
        let v = Self::binprot_read(&mut r)?;
        io::copy(&mut r, &mut io::sink())?;
        Ok(v)
    }
}

impl<T> FromBinProtStream for T where T: BinProtRead {}

#[cfg(test)]
mod tests {
    use crate::utils::{decode_bstr_from_slice, get_sized_slice};

    use super::{decode_int_from_slice, decode_string_from_slice};

    #[test]
    fn u8() {
        for (b, i, l) in [(b"\x00", 0_u8, 1), (b"\x7f", 0x7f, 1)] {
            assert_eq!(decode_int_from_slice(b).unwrap(), (i, l));
        }
    }

    #[test]
    fn i8() {
        for (b, i, l) in [(b"\x00", 0_i8, 1), (b"\x7f", 0x7f, 1)] {
            assert_eq!(decode_int_from_slice(b).unwrap(), (i, l));
        }
    }

    #[test]
    fn u16() {
        for (b, i, l) in [
            (&b"\x00"[..], 0_u16, 1),
            (b"\x7f", 0x7f, 1),
            (b"\xfe\x80\x00", 0x80, 3),
        ] {
            assert_eq!(decode_int_from_slice(b).unwrap(), (i, l));
        }
    }

    #[test]
    fn i16() {
        for (b, i, l) in [
            (&b"\x00"[..], 0_i16, 1),
            (b"\x7f", 0x7f, 1),
            (b"\xfe\x80\x00", 0x80, 3),
        ] {
            assert_eq!(decode_int_from_slice(b).unwrap(), (i, l));
        }
    }

    #[test]
    fn string() {
        let tests: &[(&[u8], &str, usize)] = &[
            (b"\x00", "", 1),
            (b"\x00\xff", "", 1),
            (b"\x01a", "a", 2),
            (b"\x0bsome string", "some string", 12),
        ];
        for (b, s, l) in tests {
            let (string, len) = decode_string_from_slice(b).unwrap();
            assert_eq!((string.as_str(), len), (*s, *l));
        }
    }

    #[test]
    fn bstr() {
        let tests: &[(&[u8], &[u8])] = &[
            (b"\x00", b""),
            (b"\x00\xff", b""),
            (b"\x01a", b"a"),
            (b"\x0bsome string", b"some string"),
            (b"\x0bsome string with more bytes", b"some string"),
        ];
        for (b, s) in tests {
            let bstr = decode_bstr_from_slice(b).unwrap();
            assert_eq!(bstr, *s);
        }
    }

    #[test]
    fn slice() {
        let tests: &[(&[u8], &[u8])] = &[
            (b"\x00\x00\x00\x00\x00\x00\x00\x00", b""),
            (b"\x00\x00\x00\x00\x00\x00\x00\x00\xff", b""),
            (b"\x01\x00\x00\x00\x00\x00\x00\x00\xff", b"\xff"),
        ];
        for (b, s) in tests {
            let slice = get_sized_slice(b).unwrap();
            assert_eq!(slice, *s);
        }
    }

    #[test]
    fn stream() {
        use super::FromBinProtStream;
        let tests: &[(&[u8], &[u8], usize)] = &[
            (b"\x01\x00\x00\x00\x00\x00\x00\x00\x00", b"", 9),
            (b"\x02\x00\x00\x00\x00\x00\x00\x00\x01b", b"b", 10),
            (b"\x02\x00\x00\x00\x00\x00\x00\x00\x01bcdf", b"b", 10),
            (b"\x05\x00\x00\x00\x00\x00\x00\x00\x01bcdf", b"b", 13),
        ];
        for (b, s, l) in tests {
            let mut p = *b;
            let string = crate::string::String::read_from_stream(&mut p).unwrap();
            assert_eq!((string.as_ref(), b.len() - p.len()), (*s, *l));
        }
    }
}

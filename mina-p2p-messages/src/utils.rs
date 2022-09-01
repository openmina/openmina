use std::io::Read;

use binprot::BinProtRead;

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

#[cfg(test)]
mod tests {
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
}

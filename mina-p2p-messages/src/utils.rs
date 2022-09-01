use std::io::Read;

use binprot::BinProtRead;

pub fn decode_int<T, R>(r: &mut R) -> Result<T, binprot::Error> where T: BinProtRead, R: Read {
    T::binprot_read(r)
}

pub fn decode_int_from_slice<T>(slice: &[u8]) -> Result<(T, usize), binprot::Error> where T: BinProtRead {
    let mut ptr = slice;
    Ok((decode_int(&mut ptr)?, slice.len() - ptr.len()))
}

#[cfg(test)]
mod tests {
    use super::decode_int_from_slice;

    #[test]
    fn u8() {
        for (b, i, l) in [
            (b"\x00", 0_u8, 1),
            (b"\x7f", 0x7f, 1),
        ] {
            assert_eq!(decode_int_from_slice(b).unwrap(), (i, l));
        }
    }

    #[test]
    fn i8() {
        for (b, i, l) in [
            (b"\x00", 0_i8, 1),
            (b"\x7f", 0x7f, 1),
        ] {
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
}

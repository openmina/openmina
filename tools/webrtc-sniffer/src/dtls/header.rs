use std::fmt;

use nom::{
    bytes::complete::take,
    error::{Error, ErrorKind},
    number::complete::{be_u16, be_u64, be_u8},
    Err, IResult,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
}

impl ContentType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            20 => Some(ContentType::ChangeCipherSpec),
            21 => Some(ContentType::Alert),
            22 => Some(ContentType::Handshake),
            23 => Some(ContentType::ApplicationData),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub ty: ContentType,
    pub epoch: u16,
    pub sequence_number: u64,
    pub length: u16,
    pub body: &'a [u8],
}

impl fmt::Display for Chunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Chunk {
            ty,
            epoch,
            sequence_number: seq,
            length,
            ..
        } = self;
        write!(
            f,
            "{ty:?}, epoch={epoch}, seq={seq:012x}, len={length}, data={}",
            hex::encode(self.body)
        )
    }
}

impl<'a> Chunk<'a> {
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (input, ty_byte) = be_u8(input)?;
        let ty = ContentType::from_u8(ty_byte)
            .ok_or_else(|| Err::Error(Error::new(input, ErrorKind::Alt)))?;

        let (input, legacy_record_version) = be_u16(input)?;
        if legacy_record_version != 0xFEFD {
            return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
        }

        let (input, t) = be_u64(input)?;
        let epoch = (t >> 48) as u16;
        let sequence_number = t & ((1 << 48) - 1);
        let (input, length) = be_u16(input)?;
        let (input, body) = take(length as usize)(input)?;

        let header = Chunk {
            ty,
            epoch,
            sequence_number,
            length,
            body,
        };

        Ok((input, header))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let bytes = &[
            22, // ContentType::Handshake
            0xFE, 0xFD, // legacy_record_version (0xFEFD for DTLS 1.0)
            0x00, 0x01, // epoch
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // sequence_number
            0x00, 0x03, // length
            0x00, 0x00, 0x00,
        ];

        let result = Chunk::parse(bytes);
        let (_, chunk) = result.unwrap();

        assert_eq!(chunk.ty, ContentType::Handshake);
        assert_eq!(chunk.epoch, 1);
        assert_eq!(chunk.sequence_number, 1);
        assert_eq!(chunk.length, 3);
        assert_eq!(chunk.body, [0; 3]);
    }
}

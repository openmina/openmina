use std::fmt;

use nom::{
    bytes::complete::take,
    combinator::map,
    error::{Error, ErrorKind},
    multi::many0,
    number::complete::{be_u16, be_u24, be_u8},
    Err, IResult, Parser,
};

pub struct HandshakeMessage {
    pub length: u32,
    pub message_seq: u16,
    pub fragment_offset: u32,
    pub fragment_length: u32,
    pub inner: HandshakeInner,
}

pub enum HandshakeInner {
    ClientHello(ClientHello),
    ServerHello(ServerHello),
    HelloVerifyRequest(HelloVerifyRequest),
    Certificates(Certificates),
    ServerKeyExchange(ServerKeyExchange),
    CertificateRequest(u8),
    ServerHelloDone,
    CertificateVerify(u8),
    ClientKeyExchange(ClientKeyExchange),
    Finished,
}

impl fmt::Display for HandshakeInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientHello(msg) => write!(f, "ClientHello({msg})"),
            Self::ServerHello(msg) => write!(f, "ServerHello({msg})"),
            Self::HelloVerifyRequest(msg) => write!(f, "HelloVerifyRequest({msg})"),
            Self::Certificates(msg) => write!(f, "Certificates({msg})"),
            Self::ServerKeyExchange(msg) => write!(f, "ServerKeyExchange({msg})"),
            Self::CertificateRequest(msg) => write!(f, "CertificateRequest({msg})"),
            Self::ServerHelloDone => write!(f, "ServerHelloDone"),
            Self::CertificateVerify(msg) => write!(f, "CertificateVerify({msg})"),
            Self::ClientKeyExchange(msg) => write!(f, "ClientKeyExchange({msg})"),
            Self::Finished => write!(f, "Finished"),
        }
    }
}

impl HandshakeMessage {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, discriminant) = be_u8(input)?;
        let (input, length) = be_u24(input)?;
        let (input, message_seq) = be_u16(input)?;
        let (input, fragment_offset) = be_u24(input)?;
        let (input, fragment_length) = be_u24(input)?;
        let (input, inner) = match discriminant {
            1 => map(ClientHello::parse, HandshakeInner::ClientHello).parse(input),
            2 => map(ServerHello::parse, HandshakeInner::ServerHello).parse(input),
            3 => map(
                HelloVerifyRequest::parse,
                HandshakeInner::HelloVerifyRequest,
            )
            .parse(input),
            11 => map(Certificates::parse, HandshakeInner::Certificates).parse(input),
            12 => map(ServerKeyExchange::parse, HandshakeInner::ServerKeyExchange).parse(input),
            13 => Ok((input, HandshakeInner::CertificateRequest(0))),
            14 => Ok((input, HandshakeInner::ServerHelloDone)),
            15 => Ok((input, HandshakeInner::CertificateVerify(0))),
            16 => map(ClientKeyExchange::parse, HandshakeInner::ClientKeyExchange).parse(input),
            20 => Ok((input, HandshakeInner::Finished)),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        }?;
        Ok((
            input,
            HandshakeMessage {
                length,
                message_seq,
                fragment_offset,
                fragment_length,
                inner,
            },
        ))
    }
}

pub struct ClientHello {
    pub random: [u8; 32],
    pub session_id: Vec<u8>,
    pub cookie: Vec<u8>,
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<Extension>,
}

impl fmt::Display for ClientHello {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "random={}, session_id=\"{}\", cookie=\"{}\", cipher_suites={:?}, compression_methods={:?}",
            hex::encode(&self.random),
            hex::encode(&self.session_id),
            hex::encode(&self.cookie),
            self.cipher_suites,
            self.compression_methods,
        )
    }
}

impl ClientHello {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, legacy_record_version) = be_u16(input)?;
        if legacy_record_version != 0xFEFD {
            return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
        }
        let (input, random) = take(32usize)(input)?;
        let random = <[u8; 32]>::try_from(random).expect("cannot fail");
        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let session_id = bytes.to_vec();
        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let cookie = bytes.to_vec();
        let (input, l) = be_u16(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let (_, cipher_suites) = many0(be_u16).parse(bytes)?;
        let (input, compression_methods_len) = be_u8(input)?;
        let (input, bytes) = take(compression_methods_len as usize)(input)?;
        let compression_methods = bytes.to_vec();
        let (input, l) = be_u16(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let (_, extensions) = many0(Extension::parse).parse(bytes)?;

        Ok((
            input,
            ClientHello {
                random,
                session_id,
                cookie,
                cipher_suites,
                compression_methods,
                extensions,
            },
        ))
    }
}

pub struct ServerHello {
    pub random: [u8; 32],
    pub session_id: Vec<u8>,
    pub cipher_suite: u16,
    pub compression_method: u8,
    pub extensions: Vec<Extension>,
}

impl fmt::Display for ServerHello {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "random={}, session_id=\"{}\", cipher_suite={}, compression_method={}",
            hex::encode(&self.random),
            hex::encode(&self.session_id),
            self.cipher_suite,
            self.compression_method,
        )
    }
}

impl ServerHello {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, legacy_record_version) = be_u16(input)?;
        if legacy_record_version != 0xFEFD {
            return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
        }
        let (input, random) = take(32usize)(input)?;
        let random = <[u8; 32]>::try_from(random).expect("cannot fail");
        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let session_id = bytes.to_vec();
        let (input, cipher_suite) = be_u16(input)?;
        let (input, compression_method) = be_u8(input)?;
        let (input, l) = be_u16(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let (_, extensions) = many0(Extension::parse).parse(bytes)?;

        Ok((
            input,
            ServerHello {
                random,
                session_id,
                cipher_suite,
                compression_method,
                extensions,
            },
        ))
    }
}

pub struct HelloVerifyRequest {
    pub cookie: Vec<u8>,
}

impl fmt::Display for HelloVerifyRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cookie={}", hex::encode(&self.cookie),)
    }
}

impl HelloVerifyRequest {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, legacy_record_version) = be_u16(input)?;
        if legacy_record_version != 0xFEFD {
            return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
        }

        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let cookie = bytes.to_vec();

        Ok((input, HelloVerifyRequest { cookie }))
    }
}

pub struct Extension {}

impl Extension {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let _ = be_u8(input)?;
        Ok((&[], Extension {}))
    }
}

pub struct Certificates(pub Vec<Certificate>);

impl fmt::Display for Certificates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for certificate in &self.0 {
            write!(f, "certificate({certificate})")?;
        }
        Ok(())
    }
}

impl Certificates {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, length) = be_u24(input)?;
        let (input, bytes) = take(length as usize)(input)?;
        let (_, certificates) = many0(Certificate::parse).parse(bytes)?;
        Ok((input, Certificates(certificates)))
    }
}

pub struct Certificate {
    pub data: Vec<u8>,
}

impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.data))
    }
}

impl Certificate {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, length) = be_u24(input)?;
        let (input, bytes) = take(length as usize)(input)?;
        let data = bytes.to_vec();

        Ok((input, Certificate { data }))
    }
}

pub struct ServerKeyExchange {
    // pub curve_type: u8,
    pub curve_name: u16,
    pub public_key: Vec<u8>,
    pub signature_hash_algorithm: u8,
    pub signature_algorithm: u8,
    pub signature: Vec<u8>,
}

impl fmt::Display for ServerKeyExchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "curve_name={}, pk={}, sig_alg=({},{}), sig={}",
            self.curve_name,
            hex::encode(&self.public_key),
            self.signature_hash_algorithm,
            self.signature_algorithm,
            hex::encode(&self.signature)
        )
    }
}

impl ServerKeyExchange {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, curve_type) = be_u8(input)?;
        if curve_type != 3 {
            return Err(Err::Failure(Error::new(input, ErrorKind::Alt)));
        }
        let (input, curve_name) = be_u16(input)?;

        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let public_key = bytes.to_vec();

        let (input, signature_hash_algorithm) = be_u8(input)?;
        let (input, signature_algorithm) = be_u8(input)?;

        let (input, l) = be_u16(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let signature = bytes.to_vec();

        Ok((
            input,
            ServerKeyExchange {
                curve_name,
                public_key,
                signature_hash_algorithm,
                signature_algorithm,
                signature,
            },
        ))
    }
}

pub struct ClientKeyExchange {
    pub public_key: Vec<u8>,
}

impl fmt::Display for ClientKeyExchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pk={}", hex::encode(&self.public_key),)
    }
}

impl ClientKeyExchange {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, l) = be_u8(input)?;
        let (input, bytes) = take(l as usize)(input)?;
        let public_key = bytes.to_vec();

        Ok((input, ClientKeyExchange { public_key }))
    }
}

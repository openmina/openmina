use super::keys_proto;
use crate::{
    identity::PublicKey,
    token::{self, StreamKind},
};
use multiaddr::Multiaddr;
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkIdentify {
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
    pub public_key: Option<PublicKey>,
    pub listen_addrs: Vec<Multiaddr>,
    pub observed_addr: Option<Multiaddr>,
    pub protocols: Vec<token::StreamKind>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkIdentifyFromMessageError {
    #[error("cant parse protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("unsupported public key type: {0}")]
    UnsupportedPubKeyType(String),
    #[error("error parsing public key: {0}")]
    ErrorParsingPubKey(String),
    #[error("{0}")]
    MultiaddrError(String),
}

impl TryFrom<super::pb::Identify> for P2pNetworkIdentify {
    type Error = P2pNetworkIdentifyFromMessageError;

    fn try_from(value: super::pb::Identify) -> Result<Self, Self::Error> {
        let protocol_version = value.protocol_version;
        let agent_version = value.agent_version;

        let public_key = match value.public_key {
            Some(pubkey) => Some(parse_public_key(&pubkey)?),
            None => None,
        };

        let mut listen_addrs = Vec::new();

        for addr in value.listen_addrs.iter().cloned() {
            listen_addrs.push(
                addr.try_into()
                    .map_err(|err: multiaddr::Error| err.to_string())
                    .map_err(P2pNetworkIdentifyFromMessageError::MultiaddrError)?,
            )
        }

        let observed_addr = match value.observed_addr {
            Some(addr) => Some(
                addr.try_into()
                    .map_err(|err: multiaddr::Error| err.to_string())
                    .map_err(P2pNetworkIdentifyFromMessageError::MultiaddrError)?,
            ),
            None => None,
        };

        let mut protocols = Vec::new();

        for proto in value.protocols.iter() {
            protocols.push(parse_protocol(proto)?)
        }

        Ok(Self {
            protocol_version,
            agent_version,
            public_key,
            listen_addrs,
            observed_addr,
            protocols,
        })
    }
}

impl P2pNetworkIdentify {
    pub fn to_proto_message(&self) -> super::pb::Identify {
        super::pb::Identify::from(self)
    }
}

impl<'a> From<&'a P2pNetworkIdentify> for super::pb::Identify {
    fn from(value: &'a P2pNetworkIdentify) -> Self {
        Self {
            protocol_version: value.protocol_version.as_ref().map(|v| v.into()),
            agent_version: value.agent_version.as_ref().map(|v| v.into()),
            public_key: value.public_key.as_ref().map(|key| {
                let key_bytes = key.to_bytes();
                let pubkey = keys_proto::PublicKey {
                    Type: crate::network::identify::KeyType::Ed25519,
                    Data: key_bytes.as_ref().into(),
                };
                let mut buf = Vec::with_capacity(pubkey.get_size());
                let mut writer = Writer::new(&mut buf);

                pubkey.write_message(&mut writer).expect("encoding success");
                buf
            }),
            listen_addrs: value.listen_addrs.iter().map(|v| v.to_vec()).collect(),
            observed_addr: value.observed_addr.as_ref().map(|v| v.to_vec()),
            protocols: value
                .protocols
                .iter()
                .map(|v| v.name_str().into())
                .collect(),
        }
    }
}

pub fn parse_public_key(key_bytes: &[u8]) -> Result<PublicKey, P2pNetworkIdentifyFromMessageError> {
    let mut reader = BytesReader::from_bytes(key_bytes);

    keys_proto::PublicKey::from_reader(&mut reader, key_bytes).map_or_else(
        |error| {
            Err(P2pNetworkIdentifyFromMessageError::ErrorParsingPubKey(
                error.to_string(),
            ))
        },
        |pubkey| match pubkey {
            keys_proto::PublicKey {
                Type: keys_proto::KeyType::Ed25519,
                Data: data,
            } => {
                let bytes = data[..].try_into().or(Err(
                    P2pNetworkIdentifyFromMessageError::ErrorParsingPubKey(format!(
                        "invalid size {}",
                        data.len()
                    )),
                ))?;

                PublicKey::from_bytes(bytes).map_err(|err| {
                    P2pNetworkIdentifyFromMessageError::ErrorParsingPubKey(err.to_string())
                })
            }
            _ => Err(P2pNetworkIdentifyFromMessageError::UnsupportedPubKeyType(
                format!("{:?}", pubkey.Type),
            )),
        },
    )
}

pub fn parse_protocol(name: &str) -> Result<StreamKind, P2pNetworkIdentifyFromMessageError> {
    // buffer content should match one of tokens
    for tok in token::Token::ALL.iter() {
        if let token::Token::Protocol(token::Protocol::Stream(a)) = tok {
            if a.name_str() == name {
                return Ok(*a);
            }
        }
    }

    Err(P2pNetworkIdentifyFromMessageError::UnsupportedProtocol(
        name.to_string(),
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
#[error("error decoding Multiaddr from bytes: {0}")]
pub struct P2pNetworkIdentifyMultiaddrError(String);

impl From<multiaddr::Error> for P2pNetworkIdentifyMultiaddrError {
    fn from(value: multiaddr::Error) -> Self {
        P2pNetworkIdentifyMultiaddrError(value.to_string())
    }
}

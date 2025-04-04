mod http;
pub use http::HttpSignalingInfo;

use std::{fmt, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PeerId;

#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum SignalingMethod {
    Http(HttpSignalingInfo),
    Https(HttpSignalingInfo),
    /// Proxy used as an SSL gateway to the actual signaling server.
    HttpsProxy(u16, HttpSignalingInfo),
    P2p {
        relay_peer_id: PeerId,
    },
}

impl SignalingMethod {
    pub fn can_connect_directly(&self) -> bool {
        match self {
            Self::Http(_) | Self::Https(_) | Self::HttpsProxy(_, _) => true,
            Self::P2p { .. } => false,
        }
    }

    /// If method is http or https, it will return url to which an
    /// offer can be sent.
    pub fn http_url(&self) -> Option<String> {
        let (http, info) = match self {
            Self::Http(info) => ("http", info),
            Self::Https(info) => ("https", info),
            Self::HttpsProxy(cluster_id, info) => {
                return Some(format!(
                    "https://{}:{}/clusters/{}/mina/webrtc/signal",
                    info.host, info.port, cluster_id
                ));
            }
            _ => return None,
        };
        Some(format!(
            "{http}://{}:{}/mina/webrtc/signal",
            info.host, info.port,
        ))
    }

    pub fn p2p_relay_peer_id(&self) -> Option<PeerId> {
        match self {
            Self::P2p { relay_peer_id } => Some(*relay_peer_id),
            _ => None,
        }
    }
}

impl fmt::Display for SignalingMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(signaling) => {
                write!(f, "/http")?;
                signaling.fmt(f)
            }
            Self::Https(signaling) => {
                write!(f, "/https")?;
                signaling.fmt(f)
            }
            Self::HttpsProxy(cluster_id, signaling) => {
                write!(f, "/https_proxy/{cluster_id}")?;
                signaling.fmt(f)
            }
            Self::P2p { relay_peer_id } => {
                write!(f, "/p2p/{relay_peer_id}")
            }
        }
    }
}

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum SignalingMethodParseError {
    #[error("not enough args for the signaling method")]
    NotEnoughArgs,
    #[error("unknown signaling method: `{0}`")]
    UnknownSignalingMethod(String),
    #[error("invalid cluster id")]
    InvalidClusterId,
    #[error("host parse error: {0}")]
    HostParseError(String),
    #[error("host parse error: {0}")]
    PortParseError(String),
}

impl FromStr for SignalingMethod {
    type Err = SignalingMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(SignalingMethodParseError::NotEnoughArgs);
        }

        let method_end_index = s[1..]
            .find('/')
            .map(|i| i + 1)
            .filter(|i| s.len() > *i)
            .ok_or(SignalingMethodParseError::NotEnoughArgs)?;

        let rest = &s[method_end_index..];
        match &s[1..method_end_index] {
            "http" => Ok(Self::Http(rest.parse()?)),
            "https" => Ok(Self::Https(rest.parse()?)),
            "https_proxy" => {
                let mut iter = rest.splitn(3, '/').filter(|v| !v.trim().is_empty());
                let (cluster_id, rest) = (
                    iter.next()
                        .ok_or(SignalingMethodParseError::NotEnoughArgs)?,
                    iter.next()
                        .ok_or(SignalingMethodParseError::NotEnoughArgs)?,
                );
                let cluster_id: u16 = cluster_id
                    .parse()
                    .or(Err(SignalingMethodParseError::InvalidClusterId))?;
                Ok(Self::HttpsProxy(cluster_id, rest.parse()?))
            }
            method => Err(SignalingMethodParseError::UnknownSignalingMethod(
                method.to_owned(),
            )),
        }
    }
}

impl Serialize for SignalingMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SignalingMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

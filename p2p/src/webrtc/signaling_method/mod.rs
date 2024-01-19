mod http;
pub use http::HttpSignalingInfo;

use std::{fmt, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum SignalingMethod {
    Http(HttpSignalingInfo),
    Https(HttpSignalingInfo),
}

impl SignalingMethod {
    /// If method is http or https, it will return url to which an
    /// offer can be sent.
    pub fn http_url(&self) -> Option<String> {
        let (http, info) = match self {
            Self::Http(info) => ("http", info),
            Self::Https(info) => ("https", info),
            // _ => return None,
        };
        Some(format!(
            "{http}://{}:{}/mina/webrtc/signal",
            info.host, info.port,
        ))
    }

    pub fn to_rpc_string(&self) -> String {
        match self {
            SignalingMethod::Http(info) => format!("http://{}:{}", info.host, info.port),
            SignalingMethod::Https(info) => format!("https://{}:{}", info.host, info.port),
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
        }
    }
}

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum SignalingMethodParseError {
    #[error("not enough args for the signaling method")]
    NotEnoughArgs,
    #[error("unknown signaling method: `{0}`")]
    UnknownSignalingMethod(String),
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

        match &s[1..method_end_index] {
            "http" => Ok(Self::Http(s[method_end_index..].parse()?)),
            "https" => Ok(Self::Https(s[method_end_index..].parse()?)),
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
        Ok(s.parse().map_err(|err| serde::de::Error::custom(err))?)
    }
}

use std::{fmt, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::webrtc::Host;

use super::SignalingMethodParseError;

#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct HttpSignalingInfo {
    pub host: Host,
    pub port: u16,
}

impl fmt::Display for HttpSignalingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.host, self.port)
    }
}

impl From<([u8; 4], u16)> for HttpSignalingInfo {
    fn from(value: ([u8; 4], u16)) -> Self {
        Self {
            host: Host::Ipv4(value.0.into()),
            port: value.1,
        }
    }
}

impl FromStr for HttpSignalingInfo {
    type Err = SignalingMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('/').filter(|v| !v.trim().is_empty());
        let host_str = iter
            .next()
            .ok_or(SignalingMethodParseError::NotEnoughArgs)?;
        let host = Host::from_str(host_str)
            .map_err(|err| SignalingMethodParseError::HostParseError(err.to_string()))?;

        let port = iter
            .next()
            .ok_or(SignalingMethodParseError::NotEnoughArgs)?
            .parse::<u16>()
            .map_err(|err| SignalingMethodParseError::PortParseError(err.to_string()))?;

        Ok(Self { host, port })
    }
}

impl Serialize for HttpSignalingInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for HttpSignalingInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

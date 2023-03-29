use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::SignalingMethodParseError;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct HttpSignalingInfo {
    pub host: url::Host,
    pub port: u16,
}

impl fmt::Display for HttpSignalingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.host.to_string(), self.port)
    }
}

impl FromStr for HttpSignalingInfo {
    type Err = SignalingMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('/').filter(|v| !v.trim().is_empty());
        let host_str = iter
            .next()
            .ok_or(SignalingMethodParseError::NotEnoughArgs)?;
        let host = url::Host::parse(host_str)
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
        Ok(s.parse().map_err(|err| serde::de::Error::custom(err))?)
    }
}

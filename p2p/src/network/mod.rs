mod p2p_network_actions;
pub use self::p2p_network_actions::*;

mod p2p_network_service;
pub use self::p2p_network_service::*;

mod p2p_network_state;
pub use self::p2p_network_state::P2pNetworkState;

pub mod connection;
pub use self::connection::*;

pub mod pnet;
pub use self::pnet::*;

pub mod select;
pub use self::select::*;

pub mod noise;
pub use self::noise::*;

pub use self::data::Data;
mod data {
    use std::{fmt, ops};

    use serde::{Deserialize, Serialize};

    #[derive(Clone)]
    pub struct Data(pub Box<[u8]>);

    impl Serialize for Data {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(&self.0).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Data {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let hex_str = <&'de str>::deserialize(deserializer)?;
            hex::decode(hex_str)
                .map_err(serde::de::Error::custom)
                .map(Vec::into_boxed_slice)
                .map(Self)
        }
    }

    impl fmt::Display for Data {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", hex::encode(&self.0))
        }
    }

    impl fmt::Debug for Data {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("Data").field(&self.to_string()).finish()
        }
    }

    impl From<Vec<u8>> for Data {
        fn from(value: Vec<u8>) -> Self {
            Self(value.into_boxed_slice())
        }
    }

    impl From<Box<[u8]>> for Data {
        fn from(value: Box<[u8]>) -> Self {
            Self(value)
        }
    }

    impl ops::Deref for Data {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

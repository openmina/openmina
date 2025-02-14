mod p2p_network_actions;
use malloc_size_of_derive::MallocSizeOf;
use serde::Deserialize;
use serde::Serialize;

use crate::Limit;

use self::identify::stream::P2pNetworkIdentifyStreamError;
pub use self::p2p_network_actions::*;

mod p2p_network_service;
pub use self::p2p_network_service::*;

mod p2p_network_state;
pub use self::p2p_network_state::P2pNetworkState;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_reducer;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_effects;

pub mod scheduler;
pub use self::scheduler::*;

pub mod scheduler_effectful;
pub use self::scheduler_effectful::*;

pub mod pnet;
pub use self::pnet::*;

pub mod pnet_effectful;
pub use self::pnet_effectful::*;

pub mod select;
pub use self::select::*;

pub mod noise;
pub use self::noise::*;

pub mod yamux;
use self::stream::{P2pNetworkKadIncomingStreamError, P2pNetworkKadOutgoingStreamError};
pub use self::yamux::*;

pub mod identify;

pub mod kad;
pub use self::kad::*;

pub mod pubsub;
pub use self::pubsub::*;

pub mod rpc;
pub use self::rpc::*;

pub use self::data::{Data, DataSized};
mod data {
    use std::{fmt, ops};

    use malloc_size_of_derive::MallocSizeOf;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, MallocSizeOf)]
    pub struct DataSized<const N: usize>(pub [u8; N]);

    #[derive(Clone, Default, MallocSizeOf)]
    pub struct Data(pub Box<[u8]>);

    impl Data {
        pub fn empty() -> Self {
            Self(Box::new([0; 0]))
        }
    }

    impl<const N: usize> Serialize for DataSized<N> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(self.0).serialize(serializer)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for DataSized<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let hex_str = <&'de str>::deserialize(deserializer)?;
            hex::decode(hex_str)
                .map_err(Error::custom)
                .and_then(|v| v.try_into().map_err(|_| Error::custom("wrong size")))
                .map(Self)
        }
    }

    impl<const N: usize> fmt::Display for DataSized<N> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", hex::encode(self.0))
        }
    }

    impl<const N: usize> fmt::Debug for DataSized<N> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("Data").field(&self.to_string()).finish()
        }
    }

    impl<const N: usize> From<[u8; N]> for DataSized<N> {
        fn from(value: [u8; N]) -> Self {
            Self(value)
        }
    }

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
            write!(f, "{} (len {})", hex::encode(&self.0), self.0.len())
        }
    }

    impl fmt::Debug for Data {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = if self.len() > 32 {
                let l = self.len();
                format!(
                    "{}...omitted...{} (len {})",
                    hex::encode(&self.0[..12]),
                    hex::encode(&self.0[(l - 12)..]),
                    l
                )
            } else {
                self.to_string()
            };
            f.debug_tuple("Data").field(&s).finish()
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

    impl ops::DerefMut for Data {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

/// Errors that might happen while handling protobuf messages received via a stream.
#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize, MallocSizeOf)]
pub enum P2pNetworkStreamProtobufError<T> {
    #[error("error reading message length")]
    MessageLength,
    #[error("message is too long: {0} exceeds {1}")]
    Limit(
        usize,
        #[ignore_malloc_size_of = "doesn't allocate"] Limit<usize>,
    ),
    #[error("error reading message: {0}")]
    Message(String),
    #[error("error converting protobuf message: {0}")]
    Convert(#[from] T),
}

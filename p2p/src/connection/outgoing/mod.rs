mod p2p_connection_outgoing_state;
use mina_p2p_messages::v2;
pub use p2p_connection_outgoing_state::*;

mod p2p_connection_outgoing_actions;
pub use p2p_connection_outgoing_actions::*;

mod p2p_connection_outgoing_reducer;
pub use p2p_connection_outgoing_reducer::*;

mod p2p_connection_outgoing_effects;
pub use p2p_connection_outgoing_effects::*;
use url::Host;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    webrtc::{self, HttpSignalingInfo, SignalingMethod},
    PeerId,
};

// TODO(binier): maybe move to `crate::webrtc` module
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum P2pConnectionOutgoingInitOpts {
    WebRTC {
        peer_id: PeerId,
        signaling: webrtc::SignalingMethod,
    },
    #[cfg(not(target_arch = "wasm32"))]
    LibP2P {
        peer_id: PeerId,
        maddr: libp2p::Multiaddr,
    },
}

impl P2pConnectionOutgoingInitOpts {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_libp2p(&self) -> bool {
        matches!(self, Self::LibP2P { .. })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_libp2p(&self) -> bool {
        false
    }

    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::WebRTC { peer_id, .. } => peer_id,
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P { peer_id, .. } => peer_id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::WebRTC { .. } => "webrtc",
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P { .. } => "libp2p",
        }
    }

    /// The OCaml implementation of Mina uses the `get_some_initial_peers` RPC to exchange peer information.
    /// Try to convert this RPC response into our peer address representation.
    /// Recognize a hack for marking the webrtc signaling server. Prefixes "http://" or "https://" are schemas that indicates the host is webrtc signaling.
    pub fn try_from_mina_rpc(msg: v2::NetworkPeerPeerStableV1) -> Option<Self> {
        let peer_id_str = String::try_from(&msg.peer_id.0).ok()?;
        let peer_id = peer_id_str.parse::<libp2p::PeerId>().ok()?;
        if peer_id.as_ref().code() == 0x12 {
            // the peer_id is not supported
            return None;
        }

        let host = String::try_from(&msg.host).ok()?;

        let opts = if host.contains(':') {
            let mut it = host.split(':');
            let schema = it.next()?;
            let host = it.next()?.trim_start_matches('/');
            let signaling = match schema {
                "http" => SignalingMethod::Http(HttpSignalingInfo {
                    host: Host::parse(host).ok()?,
                    port: msg.libp2p_port.as_u64() as u16,
                }),
                "https" => SignalingMethod::Https(HttpSignalingInfo {
                    host: Host::parse(host).ok()?,
                    port: msg.libp2p_port.as_u64() as u16,
                }),
                _ => return None,
            };
            Self::WebRTC {
                peer_id: peer_id.into(),
                signaling,
            }
        } else {
            Self::LibP2P {
                peer_id: peer_id.into(),
                maddr: {
                    use libp2p::{multiaddr::Protocol, Multiaddr};
                    use std::net::IpAddr;

                    let mut a = Multiaddr::from(host.parse::<IpAddr>().ok()?);
                    a.push(Protocol::Tcp(msg.libp2p_port.0 as u16));
                    a.push(Protocol::P2p(peer_id.into()));
                    a
                },
            }
        };
        Some(opts)
    }

    /// Try to convert our peer address representation into mina RPC response.
    /// Use a hack to mark the webrtc signaling server. Add "http://" or "https://" schema to the host address.
    /// The OCaml node will recognize this address as incorrect and ignore it.
    pub fn try_into_mina_rpc(&self) -> Option<v2::NetworkPeerPeerStableV1> {
        use libp2p::{multiaddr::Protocol, PeerId};

        match self {
            P2pConnectionOutgoingInitOpts::LibP2P { peer_id, maddr } => {
                let host = maddr.iter().find_map(|protocol| match protocol {
                    Protocol::Ip4(ip) => Some(ip.to_string().into()),
                    Protocol::Ip6(ip) => Some(ip.to_string().into()),
                    Protocol::Dns(host) => Some(host),
                    Protocol::Dns4(host) => Some(host),
                    Protocol::Dns6(host) => Some(host),
                    _ => None,
                })?;
                let libp2p_port = maddr.iter().find_map(|protocol| match protocol {
                    Protocol::Tcp(v) => Some(v),
                    _ => None,
                })?;
                Some(v2::NetworkPeerPeerStableV1 {
                    host: host.as_bytes().into(),
                    libp2p_port: (libp2p_port as u64).into(),
                    peer_id: v2::NetworkPeerPeerIdStableV1(
                        PeerId::from(*peer_id).to_string().into_bytes().into(),
                    ),
                })
            }
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling } => match signaling {
                SignalingMethod::Http(info) => Some(v2::NetworkPeerPeerStableV1 {
                    host: format!("http://{}", info.host).as_bytes().into(),
                    libp2p_port: (info.port as u64).into(),
                    peer_id: v2::NetworkPeerPeerIdStableV1(
                        PeerId::from(*peer_id).to_string().into_bytes().into(),
                    ),
                }),
                SignalingMethod::Https(info) => Some(v2::NetworkPeerPeerStableV1 {
                    host: format!("https://{}", info.host).as_bytes().into(),
                    libp2p_port: (info.port as u64).into(),
                    peer_id: v2::NetworkPeerPeerIdStableV1(
                        PeerId::from(*peer_id).to_string().into_bytes().into(),
                    ),
                }),
            },
        }
    }
}

impl fmt::Display for P2pConnectionOutgoingInitOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebRTC { peer_id, signaling } => {
                write!(f, "/{}{}", peer_id, signaling)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P { maddr, .. } => {
                write!(f, "{}", maddr)
            }
        }
    }
}

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingInitOptsParseError {
    #[error("not enough args for the signaling method")]
    NotEnoughArgs,
    #[error("peer id parse error: {0}")]
    PeerIdParseError(String),
    #[error("signaling method parse error: `{0}`")]
    SignalingMethodParseError(webrtc::SignalingMethodParseError),
    #[error("other error: {0}")]
    Other(String),
}

impl FromStr for P2pConnectionOutgoingInitOpts {
    type Err = P2pConnectionOutgoingInitOptsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(P2pConnectionOutgoingInitOptsParseError::NotEnoughArgs);
        }

        let is_libp2p_maddr = s.starts_with("/ip4") || s.starts_with("/dns4");
        #[cfg(not(target_arch = "wasm32"))]
        if is_libp2p_maddr {
            let maddr = libp2p::Multiaddr::from_str(s)
                .map_err(|e| P2pConnectionOutgoingInitOptsParseError::Other(e.to_string()))?;
            let hash = maddr
                .iter()
                .find_map(|p| match p {
                    libp2p::multiaddr::Protocol::P2p(hash) => Some(hash),
                    _ => None,
                })
                .ok_or(P2pConnectionOutgoingInitOptsParseError::Other(
                    "peer_id not set in multiaddr. Missing `../p2p/<peer_id>`".to_string(),
                ))?;
            let peer_id = libp2p::PeerId::from_multihash(hash).map_err(|_| {
                P2pConnectionOutgoingInitOptsParseError::Other(
                    "invalid peer_id multihash".to_string(),
                )
            })?;
            return Ok(Self::LibP2P {
                peer_id: peer_id.into(),
                maddr,
            });
        }
        #[cfg(target_arch = "wasm32")]
        if is_libp2p_maddr {
            return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                "libp2p not supported in wasm".to_owned(),
            ));
        }

        let id_end_index = s[1..]
            .find('/')
            .map(|i| i + 1)
            .filter(|i| s.len() > *i)
            .ok_or(P2pConnectionOutgoingInitOptsParseError::NotEnoughArgs)?;

        Ok(Self::WebRTC {
            peer_id: s[1..id_end_index].parse::<PeerId>().map_err(|err| {
                P2pConnectionOutgoingInitOptsParseError::PeerIdParseError(err.to_string())
            })?,
            signaling: s[id_end_index..]
                .parse::<webrtc::SignalingMethod>()
                .map_err(|err| {
                    P2pConnectionOutgoingInitOptsParseError::SignalingMethodParseError(err)
                })?,
        })
    }
}

impl Serialize for P2pConnectionOutgoingInitOpts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for P2pConnectionOutgoingInitOpts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(s.parse().map_err(|err| serde::de::Error::custom(err))?)
    }
}

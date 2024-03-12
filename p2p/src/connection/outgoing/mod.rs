mod p2p_connection_outgoing_state;
pub use p2p_connection_outgoing_state::*;

mod p2p_connection_outgoing_actions;
pub use p2p_connection_outgoing_actions::*;

mod p2p_connection_outgoing_reducer;

mod p2p_connection_outgoing_effects;

use std::net::SocketAddr;
use std::{fmt, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use mina_p2p_messages::v2;

use crate::{webrtc, PeerId};

use crate::webrtc::{HttpSignalingInfo, SignalingMethod};

#[cfg(not(target_arch = "wasm32"))]
use crate::webrtc::Host;

// TODO(binier): maybe move to `crate::webrtc` module
#[derive(
    BinProtWrite, BinProtRead, derive_more::From, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub enum P2pConnectionOutgoingInitOpts {
    WebRTC {
        peer_id: PeerId,
        signaling: webrtc::SignalingMethod,
    },
    #[cfg(not(target_arch = "wasm32"))]
    LibP2P(P2pConnectionOutgoingInitLibp2pOpts),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct P2pConnectionOutgoingInitLibp2pOpts {
    pub peer_id: PeerId,
    pub host: Host,
    pub port: u16,
}

#[cfg(not(target_arch = "wasm32"))]
mod libp2p_opts {
    use std::net::SocketAddr;

    use multiaddr::Multiaddr;

    use crate::{webrtc::Host, PeerId};

    impl super::P2pConnectionOutgoingInitLibp2pOpts {
        fn to_peer_id_multiaddr(&self) -> (PeerId, Multiaddr) {
            (
                self.peer_id.clone(),
                Multiaddr::from_iter([(&self.host).into(), multiaddr::Protocol::Tcp(self.port)]),
            )
        }
        fn into_peer_id_multiaddr(self) -> (PeerId, Multiaddr) {
            (
                self.peer_id,
                Multiaddr::from_iter([(&self.host).into(), multiaddr::Protocol::Tcp(self.port)]),
            )
        }
    }

    impl From<&super::P2pConnectionOutgoingInitLibp2pOpts> for (PeerId, Multiaddr) {
        fn from(value: &super::P2pConnectionOutgoingInitLibp2pOpts) -> Self {
            value.to_peer_id_multiaddr()
        }
    }

    impl From<super::P2pConnectionOutgoingInitLibp2pOpts> for (PeerId, Multiaddr) {
        fn from(value: super::P2pConnectionOutgoingInitLibp2pOpts) -> Self {
            value.into_peer_id_multiaddr()
        }
    }

    impl From<(PeerId, SocketAddr)> for super::P2pConnectionOutgoingInitLibp2pOpts {
        fn from((peer_id, addr): (PeerId, SocketAddr)) -> Self {
            let (host, port) = match addr {
                SocketAddr::V4(v4) => (Host::Ipv4(*v4.ip()), v4.port()),
                SocketAddr::V6(v6) => (Host::Ipv6(*v6.ip()), v6.port()),
            };
            super::P2pConnectionOutgoingInitLibp2pOpts {
                peer_id,
                host,
                port,
            }
        }
    }
}

impl P2pConnectionOutgoingInitOpts {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_libp2p(&self) -> bool {
        matches!(self, Self::LibP2P(_))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_libp2p(&self) -> bool {
        false
    }

    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::WebRTC { peer_id, .. } => peer_id,
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P(v) => &v.peer_id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::WebRTC { .. } => "webrtc",
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P(_) => "libp2p",
        }
    }

    /// The OCaml implementation of Mina uses the `get_some_initial_peers` RPC to exchange peer information.
    /// Try to convert this RPC response into our peer address representation.
    /// Recognize a hack for marking the webrtc signaling server.
    /// Prefixes "http://" or "https://" are schemas that indicates the host is webrtc signaling.
    pub fn try_from_mina_rpc(msg: v2::NetworkPeerPeerStableV1) -> Option<Self> {
        let peer_id_str = String::try_from(&msg.peer_id.0).ok()?;
        let peer_id = peer_id_str.parse::<libp2p_identity::PeerId>().ok()?;
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
                    host: host.parse().ok()?,
                    port: msg.libp2p_port.as_u64() as u16,
                }),
                "https" => SignalingMethod::Https(HttpSignalingInfo {
                    host: host.parse().ok()?,
                    port: msg.libp2p_port.as_u64() as u16,
                }),
                _ => return None,
            };
            Self::WebRTC {
                peer_id: peer_id.into(),
                signaling,
            }
        } else {
            let opts = P2pConnectionOutgoingInitLibp2pOpts {
                peer_id: peer_id.into(),
                host: host.parse().ok()?,
                port: msg.libp2p_port.as_u64() as u16,
            };
            Self::LibP2P(opts)
        };
        Some(opts)
    }

    /// Try to convert our peer address representation into mina RPC response.
    /// Use a hack to mark the webrtc signaling server. Add "http://" or "https://" schema to the host address.
    /// The OCaml node will recognize this address as incorrect and ignore it.
    pub fn try_into_mina_rpc(&self) -> Option<v2::NetworkPeerPeerStableV1> {
        match self {
            P2pConnectionOutgoingInitOpts::LibP2P(opts) => Some(v2::NetworkPeerPeerStableV1 {
                host: opts.host.to_string().as_bytes().into(),
                libp2p_port: (opts.port as u64).into(),
                peer_id: v2::NetworkPeerPeerIdStableV1(
                    libp2p_identity::PeerId::from(opts.peer_id)
                        .to_string()
                        .into_bytes()
                        .into(),
                ),
            }),
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

    pub fn from_libp2p_socket_addr(peer_id: PeerId, addr: SocketAddr) -> Self {
        P2pConnectionOutgoingInitOpts::LibP2P((peer_id, addr).into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl P2pConnectionOutgoingInitLibp2pOpts {
    pub fn to_maddr(&self) -> multiaddr::Multiaddr {
        self.into()
    }
}

impl fmt::Display for P2pConnectionOutgoingInitOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebRTC { peer_id, signaling } => {
                write!(f, "/{}{}", peer_id, signaling)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::LibP2P(v) => {
                write!(f, "{}", v.to_maddr())
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

        let is_libp2p_maddr = s.starts_with("/ip") || s.starts_with("/dns");

        #[cfg(not(target_arch = "wasm32"))]
        if is_libp2p_maddr {
            let maddr = multiaddr::Multiaddr::from_str(s)
                .map_err(|e| P2pConnectionOutgoingInitOptsParseError::Other(e.to_string()))?;

            let opts = (&maddr).try_into()?;

            return Ok(Self::LibP2P(opts));
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

#[cfg(not(target_arch = "wasm32"))]
impl From<&P2pConnectionOutgoingInitLibp2pOpts> for multiaddr::Multiaddr {
    fn from(value: &P2pConnectionOutgoingInitLibp2pOpts) -> Self {
        use multiaddr::Protocol;

        Self::empty()
            .with(match &value.host {
                // maybe should be just `Dns`?
                Host::Domain(v) => Protocol::Dns4(v.into()),
                Host::Ipv4(v) => Protocol::Ip4(*v),
                Host::Ipv6(v) => Protocol::Ip6(*v),
            })
            .with(Protocol::Tcp(value.port))
            .with(Protocol::P2p(
                libp2p_identity::PeerId::from(value.peer_id).into(),
            ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<&multiaddr::Multiaddr> for P2pConnectionOutgoingInitOpts {
    type Error = P2pConnectionOutgoingInitOptsParseError;

    fn try_from(value: &multiaddr::Multiaddr) -> Result<Self, Self::Error> {
        Ok(Self::LibP2P(value.try_into()?))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<&multiaddr::Multiaddr> for P2pConnectionOutgoingInitLibp2pOpts {
    type Error = P2pConnectionOutgoingInitOptsParseError;

    fn try_from(maddr: &multiaddr::Multiaddr) -> Result<Self, Self::Error> {
        use multiaddr::Protocol;

        let mut iter = maddr.iter();
        Ok(P2pConnectionOutgoingInitLibp2pOpts {
            host: match iter.next() {
                Some(Protocol::Ip4(v)) => Host::Ipv4(v),
                Some(Protocol::Dns(v) | Protocol::Dns4(v) | Protocol::Dns6(v)) => {
                    Host::Domain(v.into_owned())
                }
                Some(_) => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "unexpected part in multiaddr! expected host".to_string(),
                    ));
                }
                None => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "missing host part from multiaddr".to_string(),
                    ));
                }
            },
            port: match iter.next() {
                Some(Protocol::Tcp(port)) => port,
                Some(_) => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "unexpected part in multiaddr! expected port".to_string(),
                    ));
                }
                None => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "missing port part from multiaddr".to_string(),
                    ));
                }
            },
            peer_id: match iter.next() {
                Some(Protocol::P2p(hash)) => libp2p_identity::PeerId::from_multihash(hash.into())
                    .map_err(|_| {
                        P2pConnectionOutgoingInitOptsParseError::Other(
                            "invalid peer_id multihash".to_string(),
                        )
                    })?
                    .into(),
                Some(_) => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "unexpected part in multiaddr! expected peer_id".to_string(),
                    ));
                }
                None => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "peer_id not set in multiaddr. Missing `../p2p/<peer_id>`".to_string(),
                    ));
                }
            },
        })
    }
}

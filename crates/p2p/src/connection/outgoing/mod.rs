mod p2p_connection_outgoing_state;
pub use p2p_connection_outgoing_state::*;

mod p2p_connection_outgoing_actions;
pub use p2p_connection_outgoing_actions::*;

mod p2p_connection_outgoing_reducer;

#[cfg(feature = "p2p-libp2p")]
use std::net::SocketAddr;
use std::{fmt, net::IpAddr, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "p2p-libp2p")]
use mina_p2p_messages::v2;

use crate::{
    webrtc::{self, Host, HttpSignalingInfo},
    PeerId,
};

#[cfg(feature = "p2p-libp2p")]
use crate::webrtc::SignalingMethod;

// TODO(binier): maybe move to `crate::webrtc` module
#[derive(
    BinProtWrite, BinProtRead, derive_more::From, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub enum P2pConnectionOutgoingInitOpts {
    WebRTC {
        peer_id: PeerId,
        signaling: webrtc::SignalingMethod,
    },
    LibP2P(P2pConnectionOutgoingInitLibp2pOpts),
}

impl P2pConnectionOutgoingInitOpts {
    pub fn with_host_resolved(self) -> Option<Self> {
        if let Self::LibP2P(libp2p_opts) = self {
            Some(Self::LibP2P(libp2p_opts.with_host_resolved()?))
        } else {
            Some(self)
        }
    }
}

#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct P2pConnectionOutgoingInitLibp2pOpts {
    pub peer_id: PeerId,
    pub host: Host,
    pub port: u16,
}

impl P2pConnectionOutgoingInitLibp2pOpts {
    /// If the current host is local and there is a better host among the `addrs`,
    /// replace the current one with the better one.
    pub fn update_host_if_needed<'a>(&mut self, mut addrs: impl Iterator<Item = &'a Multiaddr>) {
        fn is_local(ip: impl Into<IpAddr>) -> bool {
            match ip.into() {
                IpAddr::V4(ip) => ip.is_loopback() || ip.is_private(),
                IpAddr::V6(ip) => ip.is_loopback(),
            }
        }

        // if current dial opts is not good enough
        let update = match &self.host {
            Host::Domain(_) => false,
            Host::Ipv4(ip) => is_local(*ip),
            Host::Ipv6(ip) => is_local(*ip),
        };
        if update {
            // if new options is better
            let new = addrs.find_map(|x| {
                x.iter().find_map(|x| match x {
                    Protocol::Dns4(hostname) | Protocol::Dns6(hostname) => {
                        Some(Host::Domain(hostname.into_owned()))
                    }
                    Protocol::Ip4(ip) if !is_local(ip) => Some(Host::Ipv4(ip)),
                    Protocol::Ip6(ip) if !is_local(ip) => Some(Host::Ipv6(ip)),
                    _ => None,
                })
            });
            if let Some(new) = new {
                self.host = new;
            }
        }
    }

    pub fn with_host_resolved(mut self) -> Option<Self> {
        self.host = self.host.resolve()?;
        Some(self)
    }
}

pub(crate) mod libp2p_opts {
    use std::net::{IpAddr, SocketAddr};

    use multiaddr::Multiaddr;

    use crate::{webrtc::Host, PeerId};

    impl super::P2pConnectionOutgoingInitLibp2pOpts {
        fn to_peer_id_multiaddr(&self) -> (PeerId, Multiaddr) {
            (
                self.peer_id,
                Multiaddr::from_iter([(&self.host).into(), multiaddr::Protocol::Tcp(self.port)]),
            )
        }
        fn into_peer_id_multiaddr(self) -> (PeerId, Multiaddr) {
            (
                self.peer_id,
                Multiaddr::from_iter([(&self.host).into(), multiaddr::Protocol::Tcp(self.port)]),
            )
        }

        pub fn matches_socket_addr(&self, addr: SocketAddr) -> bool {
            self.port == addr.port() && self.matches_socket_ip(addr)
        }

        pub fn matches_socket_ip(&self, addr: SocketAddr) -> bool {
            match (&self.host, addr) {
                (Host::Ipv4(ip), SocketAddr::V4(addr)) => ip == addr.ip(),
                (Host::Ipv6(ip), SocketAddr::V6(addr)) => ip == addr.ip(),
                _ => false,
            }
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

    #[derive(Debug, thiserror::Error)]
    pub enum P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError {
        #[error("name unresolved: {0}")]
        Unresolved(String),
    }

    impl TryFrom<&super::P2pConnectionOutgoingInitLibp2pOpts> for SocketAddr {
        type Error = P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError;

        fn try_from(
            value: &super::P2pConnectionOutgoingInitLibp2pOpts,
        ) -> Result<Self, Self::Error> {
            match &value.host {
                Host::Domain(name) => Err(
                    P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError::Unresolved(
                        name.clone(),
                    ),
                ),
                Host::Ipv4(ip) => Ok(SocketAddr::new(IpAddr::V4(*ip), value.port)),
                Host::Ipv6(ip) => Ok(SocketAddr::new(IpAddr::V6(*ip), value.port)),
            }
        }
    }
}

impl P2pConnectionOutgoingInitOpts {
    pub fn is_libp2p(&self) -> bool {
        matches!(self, Self::LibP2P(_))
    }

    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::WebRTC { peer_id, .. } => peer_id,
            Self::LibP2P(v) => &v.peer_id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::WebRTC { .. } => "webrtc",

            Self::LibP2P(_) => "libp2p",
        }
    }

    pub fn can_connect_directly(&self) -> bool {
        match self {
            Self::LibP2P(..) => true,
            Self::WebRTC { signaling, .. } => signaling.can_connect_directly(),
        }
    }

    pub fn webrtc_p2p_relay_peer_id(&self) -> Option<PeerId> {
        match self {
            Self::WebRTC { signaling, .. } => signaling.p2p_relay_peer_id(),
            _ => None,
        }
    }

    /// The OCaml implementation of Mina uses the `get_some_initial_peers` RPC to exchange peer information.
    /// Try to convert this RPC response into our peer address representation.
    /// Recognize a hack for marking the webrtc signaling server.
    /// Prefixes "http://" or "https://" are schemas that indicates the host is webrtc signaling.
    #[cfg(feature = "p2p-libp2p")]
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
                peer_id: peer_id.try_into().ok()?,
                signaling,
            }
        } else {
            let opts = P2pConnectionOutgoingInitLibp2pOpts {
                peer_id: peer_id.try_into().ok()?,
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
    #[cfg(feature = "p2p-libp2p")]
    pub fn try_into_mina_rpc(&self) -> Option<v2::NetworkPeerPeerStableV1> {
        match self {
            P2pConnectionOutgoingInitOpts::LibP2P(opts) => Some(v2::NetworkPeerPeerStableV1 {
                host: opts.host.to_string().as_bytes().into(),
                libp2p_port: (opts.port as u64).into(),
                peer_id: v2::NetworkPeerPeerIdStableV1(
                    libp2p_identity::PeerId::try_from(opts.peer_id)
                        .ok()?
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
                        (*peer_id).to_string().into_bytes().into(),
                    ),
                }),
                SignalingMethod::Https(info) => Some(v2::NetworkPeerPeerStableV1 {
                    host: format!("https://{}", info.host).as_bytes().into(),
                    libp2p_port: (info.port as u64).into(),
                    peer_id: v2::NetworkPeerPeerIdStableV1(
                        (*peer_id).to_string().into_bytes().into(),
                    ),
                }),
                SignalingMethod::HttpsProxy(cluster_id, info) => {
                    Some(v2::NetworkPeerPeerStableV1 {
                        host: format!("https://{}/clusters/{cluster_id}", info.host)
                            .as_bytes()
                            .into(),
                        libp2p_port: (info.port as u64).into(),
                        peer_id: v2::NetworkPeerPeerIdStableV1(
                            (*peer_id).to_string().into_bytes().into(),
                        ),
                    })
                }
                SignalingMethod::Proxied(scheme, path_prefix, info) => {
                    Some(v2::NetworkPeerPeerStableV1 {
                        host: format!("{scheme}://{}{}", info.host, path_prefix)
                            .as_bytes()
                            .into(),
                        libp2p_port: (info.port as u64).into(),
                        peer_id: v2::NetworkPeerPeerIdStableV1(
                            (*peer_id).to_string().into_bytes().into(),
                        ),
                    })
                }
                SignalingMethod::P2p { .. } => None,
            },
        }
    }

    #[cfg(feature = "p2p-libp2p")]
    pub fn from_libp2p_socket_addr(peer_id: PeerId, addr: SocketAddr) -> Self {
        P2pConnectionOutgoingInitOpts::LibP2P((peer_id, addr).into())
    }
}

impl P2pConnectionOutgoingInitLibp2pOpts {
    pub fn to_maddr(&self) -> Option<multiaddr::Multiaddr> {
        self.clone().try_into().ok()
    }
}

impl fmt::Display for P2pConnectionOutgoingInitOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maddr: Multiaddr = self.into();
        write!(f, "{}", maddr)
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

        // Try parsing as multiaddr first (the preferred format)
        if let Ok(maddr) = Multiaddr::from_str(s) {
            return Self::try_from(&maddr);
        }

        // Fallback: try legacy WebRTC format (/{peer_id}/{signaling_method}/...)
        // This format is deprecated; prefer multiaddr format instead.
        let id_end_index = s[1..]
            .find('/')
            .map(|i| i + 1)
            .filter(|i| s.len() > *i)
            .ok_or(P2pConnectionOutgoingInitOptsParseError::NotEnoughArgs)?;

        let opts = Self::WebRTC {
            peer_id: s[1..id_end_index].parse::<PeerId>().map_err(|err| {
                P2pConnectionOutgoingInitOptsParseError::PeerIdParseError(err.to_string())
            })?,
            signaling: s[id_end_index..]
                .parse::<webrtc::SignalingMethod>()
                .map_err(|err| {
                    P2pConnectionOutgoingInitOptsParseError::SignalingMethodParseError(err)
                })?,
        };

        // Emit deprecation warning with the suggested multiaddr format
        let suggested_maddr: Multiaddr = (&opts).into();
        tracing::warn!(
            message = "Deprecated address format detected. Please use multiaddr format instead.",
            legacy_format = %s,
            suggested_format = %suggested_maddr,
        );

        Ok(opts)
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
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<P2pConnectionOutgoingInitLibp2pOpts> for multiaddr::Multiaddr {
    type Error = libp2p_identity::DecodingError;

    fn try_from(value: P2pConnectionOutgoingInitLibp2pOpts) -> Result<Self, Self::Error> {
        use multiaddr::Protocol;

        Ok(Self::empty()
            .with(match &value.host {
                // maybe should be just `Dns`?
                Host::Domain(v) => Protocol::Dns4(v.into()),
                Host::Ipv4(v) => Protocol::Ip4(*v),
                Host::Ipv6(v) => Protocol::Ip6(*v),
            })
            .with(Protocol::Tcp(value.port))
            .with(Protocol::P2p(libp2p_identity::PeerId::try_from(
                value.peer_id,
            )?)))
    }
}

impl TryFrom<&multiaddr::Multiaddr> for P2pConnectionOutgoingInitOpts {
    type Error = P2pConnectionOutgoingInitOptsParseError;

    /// Parses a multiaddr into connection options.
    ///
    /// Supports both WebRTC and LibP2P multiaddr formats:
    ///
    /// **WebRTC formats** (contain `/webrtc` protocol):
    /// - HTTP(S): `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/webrtc/<http|https>/p2p/{peer_id}`
    /// - Proxied: `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/webrtc/<http|https>/http-path/{path}/p2p/{peer_id}`
    /// - P2P Relay: `/p2p/{relay_peer_id}/webrtc/p2p-circuit/p2p/{target_peer_id}`
    ///
    /// **LibP2P format**:
    /// - `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/p2p/{peer_id}`
    fn try_from(maddr: &multiaddr::Multiaddr) -> Result<Self, Self::Error> {
        // Check if this is a WebRTC multiaddr
        let is_webrtc = maddr.iter().any(|p| p == Protocol::WebRTC);

        if is_webrtc {
            let mut iter = maddr.iter().peekable();

            // Check for P2P relay format: /p2p/{relay}/webrtc/p2p-circuit/p2p/{target}
            if matches!(iter.peek(), Some(Protocol::P2p(_))) {
                let relay_peer_id = match iter.next() {
                    Some(Protocol::P2p(hash)) => {
                        libp2p_identity::PeerId::from_multihash(hash.into())
                            .map_err(|_| {
                                P2pConnectionOutgoingInitOptsParseError::Other(
                                    "invalid relay peer_id multihash".to_string(),
                                )
                            })?
                            .try_into()
                            .map_err(|_| {
                                P2pConnectionOutgoingInitOptsParseError::Other(
                                    "unexpected error converting relay PeerId".to_string(),
                                )
                            })?
                    }
                    _ => {
                        return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                            "expected p2p protocol for relay".to_string(),
                        ))
                    }
                };

                // Skip /webrtc
                match iter.next() {
                    Some(Protocol::WebRTC) => {}
                    _ => {
                        return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                            "expected webrtc protocol".to_string(),
                        ))
                    }
                }

                // Expect /p2p-circuit
                match iter.next() {
                    Some(Protocol::P2pCircuit) => {}
                    _ => {
                        return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                            "expected p2p-circuit protocol".to_string(),
                        ))
                    }
                }

                // Get target peer_id
                let peer_id = match iter.next() {
                    Some(Protocol::P2p(hash)) => {
                        libp2p_identity::PeerId::from_multihash(hash.into())
                            .map_err(|_| {
                                P2pConnectionOutgoingInitOptsParseError::Other(
                                    "invalid target peer_id multihash".to_string(),
                                )
                            })?
                            .try_into()
                            .map_err(|_| {
                                P2pConnectionOutgoingInitOptsParseError::Other(
                                    "unexpected error converting target PeerId".to_string(),
                                )
                            })?
                    }
                    _ => {
                        return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                            "expected p2p protocol for target peer".to_string(),
                        ))
                    }
                };

                return Ok(Self::WebRTC {
                    peer_id,
                    signaling: webrtc::SignalingMethod::P2p { relay_peer_id },
                });
            }

            // HTTP(S) signaling format: /dns4|ip4/{host}/tcp/{port}/webrtc/http|https/p2p/{peer_id}
            let host = match iter.next() {
                Some(Protocol::Ip4(v)) => Host::Ipv4(v),
                Some(Protocol::Ip6(v)) => Host::Ipv6(v),
                Some(Protocol::Dns(v) | Protocol::Dns4(v) | Protocol::Dns6(v)) => {
                    Host::Domain(v.to_string())
                }
                _ => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "expected host (ip4/ip6/dns) in webrtc multiaddr".to_string(),
                    ))
                }
            };

            let port = match iter.next() {
                Some(Protocol::Tcp(p)) => p,
                _ => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "expected tcp port in webrtc multiaddr".to_string(),
                    ))
                }
            };

            // Skip /webrtc
            match iter.next() {
                Some(Protocol::WebRTC) => {}
                _ => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "expected webrtc protocol".to_string(),
                    ))
                }
            }

            // Determine signaling method: http, https, or proxy with http-path
            let signaling_info = HttpSignalingInfo { host, port };
            let scheme = match iter.next() {
                Some(Protocol::Http) => webrtc::ProxyScheme::Http,
                Some(Protocol::Https) => webrtc::ProxyScheme::Https,
                _ => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "expected http or https protocol after webrtc".to_string(),
                    ))
                }
            };
            let signaling = if let Some(Protocol::HttpPath(path)) = iter.peek() {
                let path = path.to_string();
                iter.next(); // consume the peeked item
                webrtc::SignalingMethod::Proxied(scheme, webrtc::PathPrefix(path), signaling_info)
            } else {
                match scheme {
                    webrtc::ProxyScheme::Http => webrtc::SignalingMethod::Http(signaling_info),
                    webrtc::ProxyScheme::Https => webrtc::SignalingMethod::Https(signaling_info),
                }
            };

            // Get peer_id
            let peer_id = match iter.next() {
                Some(Protocol::P2p(hash)) => libp2p_identity::PeerId::from_multihash(hash.into())
                    .map_err(|_| {
                        P2pConnectionOutgoingInitOptsParseError::Other(
                            "invalid peer_id multihash".to_string(),
                        )
                    })?
                    .try_into()
                    .map_err(|_| {
                        P2pConnectionOutgoingInitOptsParseError::Other(
                            "unexpected error converting PeerId".to_string(),
                        )
                    })?,
                _ => {
                    return Err(P2pConnectionOutgoingInitOptsParseError::Other(
                        "expected p2p protocol with peer_id".to_string(),
                    ))
                }
            };

            Ok(Self::WebRTC { peer_id, signaling })
        } else {
            // Standard libp2p multiaddr (no /webrtc)
            #[cfg(target_arch = "wasm32")]
            {
                Err(P2pConnectionOutgoingInitOptsParseError::Other(
                    "libp2p not supported in wasm".to_owned(),
                ))
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                Ok(Self::LibP2P(maddr.try_into()?))
            }
        }
    }
}

impl TryFrom<multiaddr::Multiaddr> for P2pConnectionOutgoingInitOpts {
    type Error = P2pConnectionOutgoingInitOptsParseError;

    fn try_from(value: multiaddr::Multiaddr) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<&P2pConnectionOutgoingInitOpts> for Multiaddr {
    /// Converts connection options to a multiaddr.
    ///
    /// **WebRTC formats** :
    /// - HTTP(S): `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/webrtc/<http|https>/p2p/{peer_id}`
    /// - Proxy: `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/webrtc/<http|https>/http-path/{url_encoded_prefix}/p2p/{peer_id}`
    /// - P2P Relay: `/p2p/{relay_peer_id}/webrtc/p2p-circuit/p2p/{target_peer_id}`
    ///
    /// **LibP2P format**:
    /// - `/<dns|dns4|dns6|ip4|ip6>/{host}/tcp/{port}/p2p/{peer_id}`
    fn from(opts: &P2pConnectionOutgoingInitOpts) -> Self {
        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling } => {
                use webrtc::SignalingMethod;

                let peer_id_proto = Protocol::P2p(
                    libp2p_identity::PeerId::try_from(*peer_id).expect("valid peer_id"),
                );

                match signaling {
                    SignalingMethod::Http(info) => {
                        let host_proto = match &info.host {
                            Host::Domain(v) => Protocol::Dns4(v.into()),
                            Host::Ipv4(v) => Protocol::Ip4(*v),
                            Host::Ipv6(v) => Protocol::Ip6(*v),
                        };
                        Multiaddr::empty()
                            .with(host_proto)
                            .with(Protocol::Tcp(info.port))
                            .with(Protocol::WebRTC)
                            .with(Protocol::Http)
                            .with(peer_id_proto)
                    }
                    SignalingMethod::Https(info) => {
                        let host_proto = match &info.host {
                            Host::Domain(v) => Protocol::Dns4(v.into()),
                            Host::Ipv4(v) => Protocol::Ip4(*v),
                            Host::Ipv6(v) => Protocol::Ip6(*v),
                        };
                        Multiaddr::empty()
                            .with(host_proto)
                            .with(Protocol::Tcp(info.port))
                            .with(Protocol::WebRTC)
                            .with(Protocol::Https)
                            .with(peer_id_proto)
                    }
                    SignalingMethod::HttpsProxy(cluster_id, info) => {
                        // Convert legacy cluster_id to path format: /clusters/{id}
                        let host_proto = match &info.host {
                            Host::Domain(v) => Protocol::Dns4(v.into()),
                            Host::Ipv4(v) => Protocol::Ip4(*v),
                            Host::Ipv6(v) => Protocol::Ip6(*v),
                        };
                        let path = format!("clusters/{}", cluster_id);
                        Multiaddr::empty()
                            .with(host_proto)
                            .with(Protocol::Tcp(info.port))
                            .with(Protocol::WebRTC)
                            .with(Protocol::Https)
                            .with(Protocol::HttpPath(path.into()))
                            .with(peer_id_proto)
                    }
                    SignalingMethod::Proxied(scheme, webrtc::PathPrefix(path), info) => {
                        let host_proto = match &info.host {
                            Host::Domain(v) => Protocol::Dns4(v.into()),
                            Host::Ipv4(v) => Protocol::Ip4(*v),
                            Host::Ipv6(v) => Protocol::Ip6(*v),
                        };
                        let scheme_proto = match scheme {
                            webrtc::ProxyScheme::Http => Protocol::Http,
                            webrtc::ProxyScheme::Https => Protocol::Https,
                        };
                        Multiaddr::empty()
                            .with(host_proto)
                            .with(Protocol::Tcp(info.port))
                            .with(Protocol::WebRTC)
                            .with(scheme_proto)
                            .with(Protocol::HttpPath(path.into()))
                            .with(peer_id_proto)
                    }
                    SignalingMethod::P2p { relay_peer_id } => {
                        let relay_id_proto = Protocol::P2p(
                            libp2p_identity::PeerId::try_from(*relay_peer_id)
                                .expect("valid relay_peer_id"),
                        );
                        Multiaddr::empty()
                            .with(relay_id_proto)
                            .with(Protocol::WebRTC)
                            .with(Protocol::P2pCircuit)
                            .with(peer_id_proto)
                    }
                }
            }
            P2pConnectionOutgoingInitOpts::LibP2P(v) => v.to_maddr().expect("valid libp2p opts"),
        }
    }
}

impl From<P2pConnectionOutgoingInitOpts> for Multiaddr {
    fn from(opts: P2pConnectionOutgoingInitOpts) -> Self {
        (&opts).into()
    }
}

impl TryFrom<&multiaddr::Multiaddr> for P2pConnectionOutgoingInitLibp2pOpts {
    type Error = P2pConnectionOutgoingInitOptsParseError;

    fn try_from(maddr: &multiaddr::Multiaddr) -> Result<Self, Self::Error> {
        use multiaddr::Protocol;

        let mut iter = maddr.iter();
        Ok(P2pConnectionOutgoingInitLibp2pOpts {
            host: match iter.next() {
                Some(Protocol::Ip4(v)) => Host::Ipv4(v),
                Some(Protocol::Dns(v) | Protocol::Dns4(v) | Protocol::Dns6(v)) => {
                    Host::Domain(v.to_string())
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
                    .try_into()
                    .map_err(|_| {
                        P2pConnectionOutgoingInitOptsParseError::Other(
                            "unexpected error converting PeerId".to_string(),
                        )
                    })?,
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

mod measurement {
    use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

    use super::P2pConnectionOutgoingInitOpts;

    // `Host` may contain `String` which allocates
    // but hostname usually small, compared to `String` container size 24 bytes
    impl MallocSizeOf for P2pConnectionOutgoingInitOpts {
        fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for P2pConnectionOutgoingInitOpts multiaddr parsing
    //!
    //! Run these tests with:
    //! ```bash
    //! cargo test -p mina-p2p connection::outgoing::tests
    //! ```

    use super::*;
    use std::net::Ipv4Addr;

    // Use libp2p PeerId format for test multiaddrs
    const TEST_PEER_ID_LIBP2P: &str = "12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv";
    const TEST_RELAY_PEER_ID_LIBP2P: &str = "12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs";

    #[test]
    fn test_parse_webrtc_http_signaling_domain() {
        let maddr_str = format!(
            "/dns4/signal.example.com/tcp/8080/webrtc/http/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => match signaling {
                webrtc::SignalingMethod::Http(info) => {
                    assert_eq!(info.host, Host::Domain("signal.example.com".to_string()));
                    assert_eq!(info.port, 8080);
                }
                _ => panic!("Expected Http signaling method"),
            },
            _ => panic!("Expected WebRTC variant"),
        }
    }

    #[test]
    fn test_parse_webrtc_http_signaling_ipv4() {
        let maddr_str = format!(
            "/ip4/192.168.1.100/tcp/8080/webrtc/http/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => match signaling {
                webrtc::SignalingMethod::Http(info) => {
                    assert_eq!(info.host, Host::Ipv4(Ipv4Addr::new(192, 168, 1, 100)));
                    assert_eq!(info.port, 8080);
                }
                _ => panic!("Expected Http signaling method"),
            },
            _ => panic!("Expected WebRTC variant"),
        }
    }

    #[test]
    fn test_parse_webrtc_https_signaling() {
        let maddr_str = format!(
            "/dns4/signal.example.com/tcp/443/webrtc/https/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => match signaling {
                webrtc::SignalingMethod::Https(info) => {
                    assert_eq!(info.host, Host::Domain("signal.example.com".to_string()));
                    assert_eq!(info.port, 443);
                }
                _ => panic!("Expected Https signaling method"),
            },
            _ => panic!("Expected WebRTC variant"),
        }
    }

    #[test]
    fn test_parse_webrtc_https_proxy_with_http_path() {
        let maddr_str = format!(
            "/dns4/proxy.example.com/tcp/443/webrtc/https/http-path/cluster%2F123%2Fmina%2Fwebrtc%2Fsignal/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => match signaling {
                webrtc::SignalingMethod::Proxied(scheme, webrtc::PathPrefix(path), info) => {
                    assert_eq!(scheme, webrtc::ProxyScheme::Https);
                    assert_eq!(path, "cluster/123/mina/webrtc/signal");
                    assert_eq!(info.host, Host::Domain("proxy.example.com".to_string()));
                    assert_eq!(info.port, 443);
                }
                _ => panic!("Expected Proxied signaling method"),
            },
            _ => panic!("Expected WebRTC variant"),
        }
    }

    #[test]
    fn test_parse_webrtc_p2p_relay() {
        let maddr_str = format!(
            "/p2p/{}/webrtc/p2p-circuit/p2p/{}",
            TEST_RELAY_PEER_ID_LIBP2P, TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => match signaling {
                webrtc::SignalingMethod::P2p { .. } => {
                    // Successfully parsed as P2P relay
                }
                _ => panic!("Expected P2p signaling method"),
            },
            _ => panic!("Expected WebRTC variant"),
        }
    }

    #[test]
    fn test_parse_libp2p_multiaddr() {
        let maddr_str = format!(
            "/dns4/seed-1.devnet.gcp.o1test.net/tcp/10003/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let opts: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) => {
                assert_eq!(
                    libp2p_opts.host,
                    Host::Domain("seed-1.devnet.gcp.o1test.net".to_string())
                );
                assert_eq!(libp2p_opts.port, 10003);
            }
            _ => panic!("Expected LibP2P variant"),
        }
    }

    #[test]
    fn test_roundtrip_webrtc_http() {
        let maddr_str = format!(
            "/dns4/signal.example.com/tcp/8080/webrtc/http/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );

        // First decode: parse multiaddr string
        let opts1: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();

        // Encode: convert to Multiaddr, then to string
        let maddr: Multiaddr = (&opts1).into();
        let encoded_str = maddr.to_string();

        // Second decode: parse the encoded string
        let opts2: P2pConnectionOutgoingInitOpts = encoded_str.parse().unwrap();

        // Both decodes should match structurally
        assert_eq!(opts1, opts2);
    }

    #[test]
    fn test_roundtrip_webrtc_https() {
        let maddr_str = format!(
            "/dns4/signal.example.com/tcp/443/webrtc/https/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );

        let opts1: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();
        let maddr: Multiaddr = (&opts1).into();
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();

        assert_eq!(opts1, opts2);
    }

    #[test]
    fn test_roundtrip_webrtc_https_proxy() {
        let maddr_str = format!(
            "/dns4/proxy.example.com/tcp/443/webrtc/https/http-path/cluster%2F123/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );

        let opts1: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();
        let maddr: Multiaddr = (&opts1).into();
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();

        assert_eq!(opts1, opts2);
    }

    #[test]
    fn test_roundtrip_webrtc_p2p_relay() {
        let maddr_str = format!(
            "/p2p/{}/webrtc/p2p-circuit/p2p/{}",
            TEST_RELAY_PEER_ID_LIBP2P, TEST_PEER_ID_LIBP2P
        );

        let opts1: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();
        let maddr: Multiaddr = (&opts1).into();
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();

        assert_eq!(opts1, opts2);
    }

    #[test]
    fn test_roundtrip_libp2p() {
        let maddr_str = format!("/dns4/example.com/tcp/10003/p2p/{}", TEST_PEER_ID_LIBP2P);

        let opts1: P2pConnectionOutgoingInitOpts = maddr_str.parse().unwrap();
        let maddr: Multiaddr = (&opts1).into();
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();

        assert_eq!(opts1, opts2);
    }

    #[test]
    fn test_from_multiaddr_webrtc_http() {
        let maddr_str = format!(
            "/dns4/signal.example.com/tcp/8080/webrtc/http/p2p/{}",
            TEST_PEER_ID_LIBP2P
        );
        let maddr: Multiaddr = maddr_str.parse().unwrap();
        let opts: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();

        assert!(matches!(
            opts,
            P2pConnectionOutgoingInitOpts::WebRTC {
                signaling: webrtc::SignalingMethod::Http(_),
                ..
            }
        ));
    }

    #[test]
    fn test_to_multiaddr_webrtc_http() {
        // Create peer_id from libp2p PeerId directly
        let libp2p_peer_id: libp2p_identity::PeerId = TEST_PEER_ID_LIBP2P.parse().unwrap();
        let peer_id: PeerId = libp2p_peer_id.try_into().unwrap();

        let opts = P2pConnectionOutgoingInitOpts::WebRTC {
            peer_id,
            signaling: webrtc::SignalingMethod::Http(HttpSignalingInfo {
                host: Host::Domain("signal.example.com".to_string()),
                port: 8080,
            }),
        };

        let maddr: Multiaddr = (&opts).into();
        let maddr_str = maddr.to_string();

        assert!(maddr_str.contains("/webrtc/http/"));
        assert!(maddr_str.contains("/dns4/signal.example.com/"));
        assert!(maddr_str.contains("/tcp/8080/"));

        // Verify roundtrip
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();
        assert_eq!(opts, opts2);
    }

    #[test]
    fn test_to_multiaddr_webrtc_p2p_relay() {
        let libp2p_peer_id: libp2p_identity::PeerId = TEST_PEER_ID_LIBP2P.parse().unwrap();
        let peer_id: PeerId = libp2p_peer_id.try_into().unwrap();

        let libp2p_relay_peer_id: libp2p_identity::PeerId =
            TEST_RELAY_PEER_ID_LIBP2P.parse().unwrap();
        let relay_peer_id: PeerId = libp2p_relay_peer_id.try_into().unwrap();

        let opts = P2pConnectionOutgoingInitOpts::WebRTC {
            peer_id,
            signaling: webrtc::SignalingMethod::P2p { relay_peer_id },
        };

        let maddr: Multiaddr = (&opts).into();
        let maddr_str = maddr.to_string();

        assert!(maddr_str.contains("/webrtc/p2p-circuit/"));
        assert!(maddr_str.starts_with("/p2p/"));

        // Verify roundtrip
        let opts2: P2pConnectionOutgoingInitOpts = (&maddr).try_into().unwrap();
        assert_eq!(opts, opts2);
    }

    #[test]
    fn test_legacy_format_still_works() {
        // Use a peer_id in the legacy format (internal base58 check encoding)
        // First parse from libp2p format to get a valid internal PeerId
        let libp2p_peer_id: libp2p_identity::PeerId = TEST_PEER_ID_LIBP2P.parse().unwrap();
        let peer_id: PeerId = libp2p_peer_id.try_into().unwrap();
        let legacy_peer_id_str = peer_id.to_string();

        // Test that legacy format /{peer_id}/{signaling} still parses
        let legacy_str = format!("/{}/http/signal.example.com/8080", legacy_peer_id_str);
        let opts: P2pConnectionOutgoingInitOpts = legacy_str.parse().unwrap();

        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => {
                assert!(matches!(signaling, webrtc::SignalingMethod::Http(_)));
            }
            _ => panic!("Expected WebRTC variant"),
        }
    }
}

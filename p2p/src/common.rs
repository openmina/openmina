use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use libp2p::{Multiaddr, multiaddr::Protocol};
use mina_p2p_messages::{
    number::Int64,
    string::ByteString,
    v2::{NetworkPeerPeerIdStableV1, NetworkPeerPeerStableV1},
};
use serde::{Deserialize, Serialize};

use crate::{
    libp2p::{P2pLibP2pAddr, PeerIdTryFromProtocol},
    webrtc::{Host, HttpSignalingInfo, SignalingMethod},
    PeerId,
};

/// Printable generic peer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pGenericPeer {
    pub peer_id: PeerId,
    pub addr: P2pGenericAddr1,
}

impl Display for P2pGenericPeer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.addr {
            P2pGenericAddr1::LibP2p(addr) => Multiaddr::from(addr).with(Protocol::P2p(self.peer_id.clone().into())).fmt(f),
            P2pGenericAddr1::WebRTC(addr) => write!(f, "/{peer_id}{addr}", peer_id = self.peer_id),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum P2pGenericPeerParseError {
    #[error(transparent)]
    Multiaddr(#[from] P2pGenericPeerTryFromMultiaddrError),
    #[error("not enough args for the signaling method")]
    NotEnoughArgs,
    #[error("peer id parse error: {0}")]
    PeerIdParseError(String),
    #[error("signaling method parse error: `{0}`")]
    SignalingMethodParseError(super::webrtc::SignalingMethodParseError),
}

impl FromStr for P2pGenericPeer {
    type Err = P2pGenericPeerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _libp2p_err = match s.parse::<Multiaddr>() {
            Ok(maddr) => return Ok(maddr.try_into()?),
            Err(err) => err,
        };

        let id_end_index = s[1..]
            .find('/')
            .map(|i| i + 1)
            .filter(|i| s.len() > *i)
            .ok_or(P2pGenericPeerParseError::NotEnoughArgs)?;

        Ok(Self {
            peer_id: s[1..id_end_index]
                .parse::<PeerId>()
                .map_err(|err| P2pGenericPeerParseError::PeerIdParseError(err.to_string()))?,
            addr: P2pGenericAddr1::WebRTC(
                s[id_end_index..]
                    .parse::<super::webrtc::SignalingMethod>()
                    .map_err(|err| P2pGenericPeerParseError::SignalingMethodParseError(err))?,
            ),
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum P2pGenericPeerTryFromMultiaddrError {
    #[error(transparent)]
    Multiaddr(#[from] super::libp2p::TryFromMultiaddrError),
    #[error("missing peer id")]
    MissingPeerId,
    #[error(transparent)]
    IncorrectPeerId(#[from] PeerIdTryFromProtocol),
    #[error("extra protocol after p2p")]
    ExtraProtocols,
}

impl TryFrom<Multiaddr> for P2pGenericPeer {
    type Error = P2pGenericPeerTryFromMultiaddrError;

    fn try_from(value: Multiaddr) -> Result<Self, Self::Error> {
        let mut iter = value.into_iter();

        let addr = P2pGenericAddr1::LibP2p((&mut iter).try_into()?);

        let Some(peer_id) = iter.next() else {
            return Err(P2pGenericPeerTryFromMultiaddrError::MissingPeerId);
        };
        let peer_id = peer_id.try_into()?;
        if iter.next().is_some() {
            return Err(P2pGenericPeerTryFromMultiaddrError::ExtraProtocols);
        }
        Ok(P2pGenericPeer { peer_id, addr })
    }
}

/// Generic peer address. Allows multiple addresses for libp2p
#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, derive_more::Display)]
pub enum P2pGenericAddrs {
    #[display(
        fmt = "{}",
        r#"_0.iter().map(|a| format!("{}:{}", a.host, a.port)).collect::<Vec<_>>().join(",")"#
    )]
    LibP2p(Vec<P2pLibP2pAddr>),
    #[display(fmt = "{}", "_0.to_rpc_string()")]
    WebRTC(SignalingMethod),
}

impl P2pGenericAddrs {
    pub fn is_empty(&self) -> bool {
        match self {
            P2pGenericAddrs::LibP2p(v) => v.is_empty(),
            P2pGenericAddrs::WebRTC(_) => false,
        }
    }

    pub fn to_generic_peers<T: FromIterator<P2pGenericPeer>>(&self, peer_id: &PeerId) -> T {
        match self {
            P2pGenericAddrs::LibP2p(v) => v
                .iter()
                .cloned()
                .map(P2pGenericAddr1::LibP2p)
                .map(|addr| P2pGenericPeer {
                    peer_id: peer_id.clone(),
                    addr,
                })
                .collect(),
            P2pGenericAddrs::WebRTC(s) => std::iter::once(s.clone())
                .map(P2pGenericAddr1::WebRTC)
                .map(|addr| P2pGenericPeer {
                    peer_id: peer_id.clone(),
                    addr,
                })
                .collect(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CollectGenericPeersError {
    #[error("mixed transport used for {0}")]
    MixedTransport(PeerId),
    #[error("multiple WebRTC addresses are used for {0}")]
    MultipleWebRTC(PeerId),
}

pub fn peer_addrs_iter<I: IntoIterator<Item = P2pGenericPeer>>(
    from: I,
) -> impl Iterator<Item = Result<(PeerId, P2pGenericAddrs), CollectGenericPeersError>> {
    let mut map = BTreeMap::new();
    for peer in from {
        match (map.get_mut(&peer.peer_id), &peer.addr) {
            (None, addr) => {
                map.insert(peer.peer_id.clone(), Ok(addr.clone().into()));
            }
            (Some(Err(_)), _) => {}
            (Some(Ok(P2pGenericAddrs::LibP2p(addrs))), P2pGenericAddr1::LibP2p(addr)) => {
                if !addrs.contains(&addr) {
                    addrs.push(addr.clone());
                }
            }
            (Some(Ok(P2pGenericAddrs::WebRTC(_))), P2pGenericAddr1::WebRTC(_)) => {
                map.insert(
                    peer.peer_id.clone(),
                    Err(CollectGenericPeersError::MultipleWebRTC(
                        peer.peer_id.clone(),
                    )),
                );
            }
            (_, _) => {
                map.insert(
                    peer.peer_id.clone(),
                    Err(CollectGenericPeersError::MixedTransport(
                        peer.peer_id.clone(),
                    )),
                );
            }
        }
    }
    map.into_iter()
        .map(|(peer_id, res)| res.map(|addr| (peer_id, addr)))
}

/// Generic printable peer address.
#[derive(Serialize, Deserialize, Debug, Clone, derive_more::Display)]
pub enum P2pGenericAddr1 {
    LibP2p(P2pLibP2pAddr),
    WebRTC(SignalingMethod),
}

impl From<P2pGenericAddr1> for P2pGenericAddrs {
    fn from(value: P2pGenericAddr1) -> Self {
        match value {
            P2pGenericAddr1::LibP2p(v) => P2pGenericAddrs::LibP2p(vec![v]),
            P2pGenericAddr1::WebRTC(v) => P2pGenericAddrs::WebRTC(v),
        }
    }
}

impl From<P2pGenericAddr1> for (ByteString, Int64) {
    fn from(value: P2pGenericAddr1) -> Self {
        match value {
            P2pGenericAddr1::LibP2p(P2pLibP2pAddr { host, port }) => {
                (host.to_string().as_bytes().into(), i64::from(port).into())
            }
            P2pGenericAddr1::WebRTC(SignalingMethod::Http(v)) => (
                format!("http://{}", v.host).as_bytes().into(),
                i64::from(v.port).into(),
            ),
            P2pGenericAddr1::WebRTC(SignalingMethod::Https(v)) => (
                format!("https://{}", v.host).as_bytes().into(),
                i64::from(v.port).into(),
            ),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum P2pGenericAddrTryFromError {
    #[error("incorrect port")]
    Port,
    #[error("host string error: {0}")]
    HostString(String),
    #[error("host parse error: {0}")]
    HostParse(String),
}

impl TryFrom<(ByteString, Int64)> for P2pGenericAddr1 {
    type Error = P2pGenericAddrTryFromError;

    fn try_from((host, port): (ByteString, Int64)) -> Result<Self, Self::Error> {
        let parse_host = |host: &String, start: Option<usize>| {
            let host = start.map_or(host.as_str(), |start| {
                host.get(start..).unwrap_or("").trim_end_matches('/')
            });
            host.parse::<Host>()
                .map_err(|e| P2pGenericAddrTryFromError::HostParse(e.to_string()))
        };

        let port = port
            .0
            .try_into()
            .map_err(|_| P2pGenericAddrTryFromError::Port)?;
        let host: String = String::try_from(host)
            .map_err(|e| P2pGenericAddrTryFromError::HostString(e.to_string()))?;
        let res = if host.starts_with("http://") {
            P2pGenericAddr1::WebRTC(SignalingMethod::Http(HttpSignalingInfo {
                host: parse_host(&host, Some(7))?,
                port,
            }))
        } else if host.starts_with("https://") {
            P2pGenericAddr1::WebRTC(SignalingMethod::Https(HttpSignalingInfo {
                host: parse_host(&host, Some(8))?,
                port,
            }))
        } else {
            P2pGenericAddr1::LibP2p(P2pLibP2pAddr {
                host: parse_host(&host, None)?,
                port,
            })
        };
        Ok(res)
    }
}

impl From<P2pGenericPeer> for NetworkPeerPeerStableV1 {
    fn from(value: P2pGenericPeer) -> Self {
        let (host, libp2p_port) = value.addr.into();
        let peer_id = NetworkPeerPeerIdStableV1(value.peer_id.to_libp2p_string().as_bytes().into());
        NetworkPeerPeerStableV1 {
            host,
            libp2p_port,
            peer_id,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum P2pGenericPeerTryFrom {
    #[error(transparent)]
    Addr(#[from] P2pGenericAddrTryFromError),
    #[error("invalid peer_id string: {0}")]
    PeerIdString(String),
    #[error("error parsing peer id: {0}")]
    PeerIdParse(String),
}

impl TryFrom<NetworkPeerPeerStableV1> for P2pGenericPeer {
    type Error = P2pGenericPeerTryFrom;

    fn try_from(value: NetworkPeerPeerStableV1) -> Result<Self, Self::Error> {
        let addr = (value.host, value.libp2p_port).try_into()?;
        let peer_id = String::try_from(value.peer_id.0)
            .map_err(|e| P2pGenericPeerTryFrom::PeerIdString(e.to_string()))?;
        let peer_id = peer_id
            .parse::<libp2p::PeerId>()
            .map_err(|e| P2pGenericPeerTryFrom::PeerIdParse(e.to_string()))?;
        let peer_id = peer_id.into();
        Ok(P2pGenericPeer { peer_id, addr })
    }
}

#[cfg(test)]
mod tests {
    use crate::common::P2pGenericPeer;

    #[test]
    fn parse_libp2p_peer() {
        for peer in [
            "/ip4/127.0.0.1/tcp/11001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            "/dns4/example.com/tcp/11001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            "/2ajh5CpZCHdv7tmMrotVnLjQXuhcuCzqKosdDmvN3tNTScw2fsd/http/65.109.110.75/10000",
        ] {
            let p2ppeer = peer.parse::<P2pGenericPeer>().expect("should be parseable");
            assert_eq!(&p2ppeer.to_string(), peer);
        }
    }
}

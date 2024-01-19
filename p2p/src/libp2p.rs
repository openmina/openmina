use libp2p::{multiaddr::Protocol, Multiaddr};
use serde::{Deserialize, Serialize};

use crate::{webrtc::Host, PeerId};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, derive_more::Display)]
#[display(fmt = "{}", "Multiaddr::from(self)")]
pub struct P2pLibP2pAddr {
    pub host: Host,
    pub port: u16,
}

impl<'a> From<&'a Host> for Protocol<'a> {
    fn from(value: &'a Host) -> Self {
        match value {
            Host::Domain(dns) => Protocol::Dns4(dns.into()),
            Host::Ipv4(ip4) => Protocol::Ip4(ip4.clone()),
            Host::Ipv6(ip6) => Protocol::Ip6(ip6.clone()),
        }
    }
}

impl<'a> From<&'a P2pLibP2pAddr> for Multiaddr {
    fn from(value: &'a P2pLibP2pAddr) -> Self {
        Multiaddr::from_iter([Protocol::from(&value.host), Protocol::Tcp(value.port)])
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("unexpected protocol kind for host")]
pub struct HostTryFromProtocol(&'static str);

impl<'a> TryFrom<Protocol<'a>> for Host {
    type Error = HostTryFromProtocol;

    fn try_from(value: Protocol) -> Result<Self, Self::Error> {
        Ok(match value {
            Protocol::Dns(dns)
            | Protocol::Dns4(dns)
            | Protocol::Dns6(dns)
            | Protocol::Dnsaddr(dns) => Host::Domain(dns.into_owned()),
            Protocol::Ip4(ip4) => Host::Ipv4(ip4),
            Protocol::Ip6(ip6) => Host::Ipv6(ip6),
            _ => return Err(HostTryFromProtocol(value.tag())),
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("unexpected protocol kind for peer_id: {_0}")]
pub struct PeerIdTryFromProtocol(&'static str);

impl<'a> TryFrom<Protocol<'a>> for PeerId {
    type Error = PeerIdTryFromProtocol;

    fn try_from(value: Protocol) -> Result<Self, Self::Error> {
        Ok(match value {
            Protocol::P2p(peer_id) => peer_id.into(),
            _ => return Err(PeerIdTryFromProtocol(value.tag())),
        })
    }
}

impl From<P2pLibP2pAddr> for Multiaddr {
    fn from(value: P2pLibP2pAddr) -> Self {
        Multiaddr::from_iter([Protocol::from(&value.host), Protocol::Tcp(value.port)])
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TryFromMultiaddrError {
    #[error("missing host")]
    MissingHost,
    #[error(transparent)]
    IncorrectHost(#[from] HostTryFromProtocol),
    #[error("missing port")]
    MissingPort,
    #[error("extra protocol after tcp")]
    ExtraProtocols,
}

use libp2p::core::multiaddr::Iter as ProtocolIter;

impl<'a> TryFrom<&mut ProtocolIter<'a>> for P2pLibP2pAddr {
    type Error = TryFromMultiaddrError;

    fn try_from(iter: &mut ProtocolIter<'a>) -> Result<Self, Self::Error> {
        let Some(host) = iter.next() else {
            return Err(TryFromMultiaddrError::MissingHost);
        };
        let host: Host = host.try_into()?;
        let Some(port) = iter.next() else {
            return Err(TryFromMultiaddrError::MissingPort);
        };
        let Protocol::Tcp(port) = port else {
            return Err(TryFromMultiaddrError::MissingPort);
        };
        Ok(P2pLibP2pAddr { host, port })
    }
}

impl TryFrom<Multiaddr> for P2pLibP2pAddr {
    type Error = TryFromMultiaddrError;

    fn try_from(value: Multiaddr) -> Result<Self, Self::Error> {
        let mut iter = value.into_iter();

        let result = (&mut iter).try_into()?;

        if iter.next().is_some() {
            return Err(TryFromMultiaddrError::ExtraProtocols);
        }
        Ok(result)
    }
}

/// Libp2p peer identifier (host + port + peer ID).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pLibP2pPeer {
    peer_id: PeerId,
    addr: P2pLibP2pAddr,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum P2pLibP2pPeerTryFromMultiaddrError {
    #[error(transparent)]
    Multiaddr(#[from] TryFromMultiaddrError),
    #[error("missing peer id")]
    MissingPeerId,
    #[error(transparent)]
    IncorrectPeerId(#[from] PeerIdTryFromProtocol),
    #[error("extra protocol after p2p")]
    ExtraProtocols,
}

impl TryFrom<Multiaddr> for P2pLibP2pPeer {
    type Error = P2pLibP2pPeerTryFromMultiaddrError;

    fn try_from(value: Multiaddr) -> Result<Self, Self::Error> {
        let mut iter = value.into_iter();

        let addr = (&mut iter).try_into()?;

        let Some(peer_id) = iter.next() else {
            return Err(P2pLibP2pPeerTryFromMultiaddrError::MissingPeerId);
        };
        let peer_id = peer_id.try_into()?;
        if iter.next().is_some() {
            return Err(P2pLibP2pPeerTryFromMultiaddrError::ExtraProtocols);
        }
        Ok(P2pLibP2pPeer { peer_id, addr })
    }
}

impl From<P2pLibP2pPeer> for Multiaddr {
    fn from(value: P2pLibP2pPeer) -> Self {
        Multiaddr::from(value.addr).with(Protocol::P2p(value.peer_id.into()))
    }
}

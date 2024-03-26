use std::{
    borrow::Cow,
    net::{IpAddr, SocketAddr},
};

use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

use super::{P2pNetworkKadEntry, P2pNetworkKadEntryTryFromError};
use crate::{mod_Message::MessageType, PeerId};

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectionType {
    NotConnected,
    Connected,
    CanConnect,
    CannotConnect,
}

impl From<super::mod_Message::ConnectionType> for ConnectionType {
    fn from(value: super::mod_Message::ConnectionType) -> Self {
        match value {
            super::mod_Message::ConnectionType::NOT_CONNECTED => ConnectionType::NotConnected,
            super::mod_Message::ConnectionType::CONNECTED => ConnectionType::Connected,
            super::mod_Message::ConnectionType::CAN_CONNECT => ConnectionType::CanConnect,
            super::mod_Message::ConnectionType::CANNOT_CONNECT => ConnectionType::CannotConnect,
        }
    }
}

impl From<ConnectionType> for super::mod_Message::ConnectionType {
    fn from(value: ConnectionType) -> Self {
        match value {
            ConnectionType::NotConnected => super::mod_Message::ConnectionType::NOT_CONNECTED,
            ConnectionType::Connected => super::mod_Message::ConnectionType::CONNECTED,
            ConnectionType::CanConnect => super::mod_Message::ConnectionType::CAN_CONNECT,
            ConnectionType::CannotConnect => super::mod_Message::ConnectionType::CANNOT_CONNECT,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKademliaRpcRequest {
    FindNode { key: PeerId },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKademliaRpcReply {
    FindNode {
        closer_peers: Vec<P2pNetworkKadEntry>,
    },
}

impl P2pNetworkKademliaRpcRequest {
    pub fn find_node(key: PeerId) -> Self {
        P2pNetworkKademliaRpcRequest::FindNode { key }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkKademliaPeerIdError {
    #[error("error decoding PeerId from bytes: lenght {0} while expected 32")]
    Parse(String),
    #[error(transparent)]
    Identity(#[from] crate::identity::PeerIdFromLibp2pPeerId),
}

impl From<libp2p_identity::ParseError> for P2pNetworkKademliaPeerIdError {
    fn from(value: libp2p_identity::ParseError) -> Self {
        P2pNetworkKademliaPeerIdError::Parse(value.to_string())
    }
}

impl<'a> TryFrom<Cow<'a, [u8]>> for PeerId {
    type Error = P2pNetworkKademliaPeerIdError;

    fn try_from(value: Cow<'a, [u8]>) -> Result<Self, Self::Error> {
        peer_id_try_from_bytes(value)
    }
}

pub(super) fn peer_id_try_from_bytes(
    bytes: Cow<'_, [u8]>,
) -> Result<PeerId, P2pNetworkKademliaPeerIdError> {
    Ok((&libp2p_identity::PeerId::from_bytes(bytes.as_ref())?).try_into()?)
}

impl<'a> From<&PeerId> for Cow<'a, [u8]> {
    fn from(value: &PeerId) -> Self {
        libp2p_identity::PeerId::from(*value)
            .to_bytes()
            .into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkKademliaRpcPeerTryFromError {
    #[error(transparent)]
    PeerId(#[from] P2pNetworkKademliaPeerIdError),
    #[error(transparent)]
    Multiaddr(#[from] P2pNetworkKademliaMultiaddrError),
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
#[error("error decoding Multiaddr from bytes: {0}")]
pub struct P2pNetworkKademliaMultiaddrError(String);

pub(super) fn multiaddr_try_from_bytes(
    bytes: Cow<'_, [u8]>,
) -> Result<Multiaddr, P2pNetworkKademliaMultiaddrError> {
    Ok(bytes.into_owned().try_into()?)
}

impl From<multiaddr::Error> for P2pNetworkKademliaMultiaddrError {
    fn from(value: multiaddr::Error) -> Self {
        P2pNetworkKademliaMultiaddrError(value.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkKademliaRpcFromMessageError {
    #[error(transparent)]
    PeerId(#[from] P2pNetworkKademliaPeerIdError),
    #[error(transparent)]
    Peer(#[from] P2pNetworkKadEntryTryFromError),
    #[error("unsupported RPC kind: {0}")]
    Unsupported(String),
}

impl<'a> TryFrom<super::Message<'a>> for P2pNetworkKademliaRpcRequest {
    type Error = P2pNetworkKademliaRpcFromMessageError;

    fn try_from(value: super::Message<'a>) -> Result<Self, Self::Error> {
        match value.type_pb {
            MessageType::FIND_NODE => {
                let key = peer_id_try_from_bytes(value.key)?;
                Ok(P2pNetworkKademliaRpcRequest::FindNode { key })
            }
            _ => Err(P2pNetworkKademliaRpcFromMessageError::Unsupported(format!(
                "{:?}",
                value.type_pb
            ))),
        }
    }
}

impl<'a> TryFrom<super::Message<'a>> for P2pNetworkKademliaRpcReply {
    type Error = P2pNetworkKademliaRpcFromMessageError;

    fn try_from(value: super::Message<'a>) -> Result<Self, Self::Error> {
        match value.type_pb {
            MessageType::FIND_NODE => {
                let closer_peers = value
                    .closerPeers
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                Ok(P2pNetworkKademliaRpcReply::FindNode { closer_peers })
            }
            _ => Err(P2pNetworkKademliaRpcFromMessageError::Unsupported(format!(
                "{:?}",
                value.type_pb
            ))),
        }
    }
}

impl<'a> From<&'a P2pNetworkKademliaRpcRequest> for super::Message<'a> {
    fn from(value: &'a P2pNetworkKademliaRpcRequest) -> Self {
        match value {
            P2pNetworkKademliaRpcRequest::FindNode { key } => super::Message {
                type_pb: MessageType::FIND_NODE,
                clusterLevelRaw: 10,
                key: key.into(),
                ..Default::default()
            },
        }
    }
}

impl<'a> From<&'a P2pNetworkKademliaRpcReply> for super::Message<'a> {
    fn from(value: &'a P2pNetworkKademliaRpcReply) -> Self {
        match value {
            P2pNetworkKademliaRpcReply::FindNode { closer_peers } => super::Message {
                type_pb: MessageType::FIND_NODE,
                clusterLevelRaw: 10,
                closerPeers: closer_peers.iter().map(Into::into).collect(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SocketAddrTryFromMultiaddrError {
    #[error("no protocol with host")]
    NoHost,
    #[error("{0} is not supported as host")]
    UnsupportedHost(String),
    #[error("no protocol for port")]
    NoPort,
    #[error("{0} is not supported as a port")]
    UnsupportedPort(String),
    #[error("extra protocol {0}")]
    ExtraProtocol(String),
}

pub fn socket_addr_try_from_multiaddr(
    maddr: &Multiaddr,
) -> Result<SocketAddr, SocketAddrTryFromMultiaddrError> {
    let mut iter = maddr.iter();
    let ip_addr = match iter.next() {
        Some(multiaddr::Protocol::Ip4(ip4)) => IpAddr::V4(ip4),
        Some(multiaddr::Protocol::Ip6(ip6)) => IpAddr::V6(ip6),
        None => return Err(SocketAddrTryFromMultiaddrError::NoHost),
        Some(p) => {
            return Err(SocketAddrTryFromMultiaddrError::UnsupportedHost(
                p.to_string(),
            ))
        }
    };
    let port = match iter.next() {
        Some(multiaddr::Protocol::Tcp(port)) => port,
        None => return Err(SocketAddrTryFromMultiaddrError::NoPort),
        Some(p) => {
            return Err(SocketAddrTryFromMultiaddrError::UnsupportedPort(
                p.to_string(),
            ))
        }
    };
    if let Some(p) = iter.next() {
        return Err(SocketAddrTryFromMultiaddrError::ExtraProtocol(
            p.to_string(),
        ));
    }
    Ok((ip_addr, port).into())
}

#[cfg(test)]
pub mod tests {
    use multiaddr::Multiaddr;
    use quick_protobuf::BytesReader;

    use crate::{
        kad::p2p_network_kad_protocol::multiaddr_try_from_bytes, P2pNetworkKademliaRpcRequest,
        PeerId,
    };

    use super::peer_id_try_from_bytes;

    #[test]
    fn peer_id_from_wire() {
        let from_bytes = |bytes: &[u8]| peer_id_try_from_bytes(bytes.into());

        assert!(from_bytes(&[]).is_err());
        // assert!(from_bytes(&[0; 32]).is_ok());

        let peer_id = "2bEgBrPTzL8wov2D4Kz34WVLCxR4uCarsBmHYXWKQA5wvBQzd9H"
            .parse::<PeerId>()
            .unwrap();
        assert_eq!(
            from_bytes(&libp2p_identity::PeerId::from(peer_id).to_bytes()).unwrap(),
            peer_id
        );
    }

    #[test]
    fn multiaddr_from_wire() {
        let from_bytes = |bytes: &[u8]| multiaddr_try_from_bytes(bytes.into());

        // /ip4/182.51.100.1/tcp/10
        assert!(from_bytes(&[4, 198, 51, 100, 1, 6, 0, 10]).is_ok());
        // /dns4/google.com/tcp/443
        assert!(
            from_bytes(&[54, 10, 103, 111, 111, 103, 108, 101, 46, 99, 111, 109, 6, 1, 187])
                .is_ok()
        );

        for multiaddr in [
            "/ip4/198.51.100.1/tcp/80",
            "/dns4/ams-2.bootstrap.libp2p.io/tcp/443",
        ] {
            let multiaddr = multiaddr.parse::<Multiaddr>().unwrap();
            assert_eq!(from_bytes(&multiaddr.to_vec()).unwrap(), multiaddr);
        }
    }

    #[test]
    fn find_nodes_from_wire() {
        let input = "2c0804500a1226002408011220bcbfc53faa51a1410b7599c1e4411d5ac45ed5a1ffdc4673c1a6e2b9e9125c4d";

        let bytes = hex::decode(input).unwrap();
        let protobuf_message = BytesReader::from_bytes(&bytes)
            .read_message::<super::super::Message>(&bytes)
            .expect("should be able to decode");

        let message = super::P2pNetworkKademliaRpcRequest::try_from(protobuf_message)
            .expect("should be able to convert");

        let P2pNetworkKademliaRpcRequest::FindNode { key } = message;
        assert_eq!(
            &key.to_libp2p_string(),
            "12D3KooWNXARF5S7qTRZZuoTZwSda7XA7fBh4oz1vZadHnaFv1nL"
        );
    }

    #[test]
    fn find_nodes_from_wire_len() {
        let input = "2c0804500a1226002408011220bcbfc53faa51a1410b7599c1e4411d5ac45ed5a1ffdc4673c1a6e2b9e9125c4d";

        let bytes = hex::decode(input).unwrap();
        let from_bytes = &mut BytesReader::from_bytes(&bytes);
        let len = from_bytes.read_varint32(&bytes).unwrap();

        println!("{} {}", len, from_bytes.len());
        let protobuf_message = BytesReader::from_bytes(&bytes)
            .read_message::<super::super::Message>(&bytes)
            .expect("should be able to decode");

        let message = super::P2pNetworkKademliaRpcRequest::try_from(protobuf_message)
            .expect("should be able to convert");

        let P2pNetworkKademliaRpcRequest::FindNode { key } = message;
        assert_eq!(
            &key.to_libp2p_string(),
            "12D3KooWNXARF5S7qTRZZuoTZwSda7XA7fBh4oz1vZadHnaFv1nL"
        );
    }
}

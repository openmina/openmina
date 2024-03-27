use std::fmt;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
use std::net::{IpAddr, SocketAddr};

use derive_more::From;
use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionResponse},
    P2pListenerId, PeerId,
};

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pEvent {
    Connection(P2pConnectionEvent),
    Listen(P2pListenEvent),
    Channel(P2pChannelEvent),
    #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
    Libp2pIdentify(PeerId, libp2p::Multiaddr),
    Discovery(P2pDiscoveryEvent),
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
    MioEvent(MioEvent),
}

/// The mio service reports events.
#[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MioEvent {
    /// A new network interface was detected on the machine.
    InterfaceDetected(IpAddr),
    /// The interface is not available anymore.
    InterfaceExpired(IpAddr),

    /// The remote peer is trying to connect to us.
    IncomingConnectionIsReady { listener: SocketAddr },
    /// We accepted the connection from the remote peer.
    IncomingConnectionDidAccept(Option<SocketAddr>, Result<(), String>),
    /// The remote peer is trying to send us some data.
    IncomingDataIsReady(SocketAddr),
    /// We received the data from the remote peer.
    IncomingDataDidReceive(SocketAddr, Result<crate::Data, String>),

    /// We connected to the remote peer by the address.
    OutgoingConnectionDidConnect(SocketAddr, Result<(), String>),
    /// We sent some data to the remote peer.
    OutgoingDataDidSend(SocketAddr, Result<(), String>),

    /// The remote peer is disconnected gracefully or with an error.
    ConnectionDidClose(SocketAddr, Result<(), String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionEvent {
    OfferSdpReady(PeerId, Result<String, String>),
    AnswerSdpReady(PeerId, Result<String, String>),
    AnswerReceived(PeerId, P2pConnectionResponse),
    Finalized(PeerId, Result<(), String>),
    Closed(PeerId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenEvent {
    NewListenAddr {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    ExpiredListenAddr {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    ListenerError {
        listener_id: P2pListenerId,
        error: String,
    },
    ListenerClosed {
        listener_id: P2pListenerId,
        error: Option<String>,
    },
}

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pChannelEvent {
    Opened(PeerId, ChannelId, Result<(), String>),
    Sent(PeerId, ChannelId, MsgId, Result<(), String>),
    Received(PeerId, Result<ChannelMsg, String>),
    Libp2pSnarkReceived(PeerId, Snark, u32),
    Closed(PeerId, ChannelId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pDiscoveryEvent {
    Ready,
    DidFindPeers(Vec<PeerId>),
    DidFindPeersError(String),
    AddRoute(PeerId, Vec<P2pConnectionOutgoingInitOpts>),
}

fn res_kind<T, E>(res: &Result<T, E>) -> &'static str {
    match res {
        Err(_) => "Err",
        Ok(_) => "Ok",
    }
}

impl fmt::Display for P2pEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P2p, ")?;
        match self {
            Self::Connection(v) => v.fmt(f),
            Self::Listen(v) => v.fmt(f),
            Self::Channel(v) => v.fmt(f),
            #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
            Self::Libp2pIdentify(peer_id, _) => {
                write!(f, "Libp2pIdentify, {peer_id}")
            }
            Self::Discovery(v) => v.fmt(f),
            #[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
            Self::MioEvent(v) => v.fmt(f),
        }
    }
}

fn maddr_ip(addr: &multiaddr::Multiaddr) -> String {
    use multiaddr::Protocol;
    addr.iter()
        .find_map(|p| match p {
            Protocol::Ip4(_)
            | Protocol::Ip6(_)
            | Protocol::Dns(_)
            | Protocol::Dns4(_)
            | Protocol::Dns6(_) => Some(p.to_string()),
            _ => None,
        })
        .unwrap_or(String::new())
}

impl fmt::Display for P2pListenEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Listen, ")?;
        match self {
            P2pListenEvent::NewListenAddr { addr, .. } => {
                write!(f, "NewListenAddr, {}", maddr_ip(addr))
            }
            P2pListenEvent::ExpiredListenAddr { addr, .. } => {
                write!(f, "ExpiredListenAddr, {}", maddr_ip(addr))
            }
            P2pListenEvent::ListenerError { error, .. } => {
                write!(f, "ListenerError, {error}")
            }
            P2pListenEvent::ListenerClosed {
                error: Some(error), ..
            } => write!(f, "ListenerClosed, {error}"),
            P2pListenEvent::ListenerClosed { error: None, .. } => write!(f, "ListenerClosed"),
        }
    }
}

impl fmt::Display for P2pConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection, ")?;
        match self {
            Self::OfferSdpReady(peer_id, res) => {
                write!(f, "OfferSdpReady, {peer_id}, {}", res_kind(res))
            }
            Self::AnswerSdpReady(peer_id, res) => {
                write!(f, "AnswerSdpReady, {peer_id}, {}", res_kind(res))
            }
            Self::AnswerReceived(peer_id, ans) => match ans {
                P2pConnectionResponse::Accepted(_) => {
                    write!(f, "AnswerReceived, {peer_id}, Accepted")
                }
                P2pConnectionResponse::Rejected(reason) => {
                    write!(f, "AnswerReceived, {peer_id}, Rejected, {reason:?}")
                }
                P2pConnectionResponse::InternalError => {
                    write!(f, "AnswerReceived, {peer_id}, InternalError")
                }
            },
            Self::Finalized(peer_id, res) => write!(f, "Finalized, {peer_id}, {}", res_kind(res)),
            Self::Closed(peer_id) => write!(f, "Closed, {peer_id}"),
        }
    }
}

impl fmt::Display for P2pChannelEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::channels::best_tip::BestTipPropagationChannelMsg;
        use crate::channels::rpc::RpcChannelMsg;
        use crate::channels::snark::SnarkPropagationChannelMsg;
        use crate::channels::snark_job_commitment::SnarkJobCommitmentPropagationChannelMsg;

        write!(f, "Channel, ")?;
        match self {
            Self::Opened(peer_id, chan_id, res) => {
                write!(f, "Opened, {peer_id}, {chan_id:?}, {}", res_kind(res))
            }
            Self::Closed(peer_id, chan_id) => {
                write!(f, "Closed, {peer_id}, {chan_id:?}")
            }
            Self::Sent(peer_id, chan_id, msg_id, res) => {
                write!(
                    f,
                    "Sent, {peer_id}, {chan_id:?}, {msg_id:?}, {}",
                    res_kind(res)
                )
            }
            Self::Libp2pSnarkReceived(peer_id, snark, nonce) => {
                write!(
                    f,
                    "Libp2pSnarkReceived, {peer_id}, fee: {}, snarker: {}, job_id: {}, nonce: {nonce}",
                    snark.fee.as_u64(),
                    snark.snarker,
                    snark.job_id(),
                )
            }
            Self::Received(peer_id, res) => {
                write!(f, "Received, {peer_id}, ")?;
                let msg = match res {
                    Err(_) => return write!(f, "Err"),
                    Ok(msg) => {
                        write!(f, "{:?}, ", msg.channel_id())?;
                        msg
                    }
                };

                match msg {
                    ChannelMsg::BestTipPropagation(v) => {
                        match v {
                            BestTipPropagationChannelMsg::GetNext => write!(f, "GetNext"),
                            // TODO(binier): avoid rehashing.
                            BestTipPropagationChannelMsg::BestTip(block) => {
                                write!(f, "{}", block.hash())
                            }
                        }
                    }
                    ChannelMsg::SnarkPropagation(v) => match v {
                        SnarkPropagationChannelMsg::GetNext { limit } => {
                            write!(f, "GetNext, limit: {limit}")
                        }
                        SnarkPropagationChannelMsg::WillSend { count } => {
                            write!(f, "WillSend, count: {count}")
                        }
                        SnarkPropagationChannelMsg::Snark(snark) => write!(
                            f,
                            "Snark, fee: {}, snarker: {}, job_id: {}",
                            snark.fee.as_u64(),
                            snark.prover,
                            snark.job_id
                        ),
                    },
                    ChannelMsg::SnarkJobCommitmentPropagation(v) => match v {
                        SnarkJobCommitmentPropagationChannelMsg::GetNext { limit } => {
                            write!(f, "GetNext, limit: {limit}")
                        }
                        SnarkJobCommitmentPropagationChannelMsg::WillSend { count } => {
                            write!(f, "WillSend, count: {count}")
                        }
                        SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment) => write!(
                            f,
                            "Commitment, fee: {}, snarker: {}, job_id: {}",
                            commitment.fee.as_u64(),
                            commitment.snarker,
                            commitment.job_id
                        ),
                    },
                    ChannelMsg::Rpc(v) => match v {
                        RpcChannelMsg::Request(id, req) => {
                            write!(f, "Request, id: {id}, {req}")
                        }
                        RpcChannelMsg::Response(id, resp) => {
                            write!(f, "Response, id: {id}, ")?;
                            match resp {
                                None => write!(f, "None"),
                                Some(resp) => write!(f, "{:?}", resp.kind()),
                            }
                        }
                    },
                }
            }
        }
    }
}

impl fmt::Display for P2pDiscoveryEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready => write!(f, "DiscoveryReady"),
            Self::DidFindPeers(peers) => write!(
                f,
                "DidFindPeers: {}",
                peers
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::DidFindPeersError(description) => {
                write!(f, "DidFindPeersError: {description}",)
            }
            Self::AddRoute(peer_id, opts) => write!(
                f,
                "AddRoute, peer_id: {peer_id}, {}",
                opts.iter()
                    .map(|x| x.peer_id().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
impl fmt::Display for MioEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InterfaceDetected(ip) => write!(f, "InterfaceDetected, {ip}"),
            Self::InterfaceExpired(ip) => write!(f, "InterfaceExpired, {ip}"),
            Self::IncomingConnectionIsReady { listener } => {
                write!(f, "IncomingConnectionIsReady, {listener}")
            }
            Self::IncomingConnectionDidAccept(Some(addr), res) => {
                write!(f, "IncomingConnectionDidAccept, {addr}, {}", res_kind(res))
            }
            Self::IncomingConnectionDidAccept(None, res) => {
                write!(f, "IncomingConnectionDidAccept, unknown, {}", res_kind(res))
            }
            Self::IncomingDataIsReady(addr) => {
                write!(f, "IncomingDataIsReady, {addr}")
            }
            Self::IncomingDataDidReceive(addr, res) => {
                write!(f, "IncomingDataDidReceive, {addr}. {}", res_kind(res))
            }
            Self::OutgoingConnectionDidConnect(addr, res) => {
                write!(f, "OutgoingConnectionDidConnect, {addr}, {}", res_kind(res))
            }
            Self::OutgoingDataDidSend(addr, res) => {
                write!(f, "OutgoingDataDidSend, {addr}, {}", res_kind(res))
            }
            Self::ConnectionDidClose(addr, res) => {
                write!(f, "ConnectionDidClose, {addr}, {}", res_kind(res))
            }
        }
    }
}

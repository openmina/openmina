use std::fmt;

use derive_more::From;
use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId},
    common::P2pGenericAddrs,
    connection::webrtc::P2pConnectionWebRTCResponse,
    P2pConnectionId, P2pListenerId, PeerId,
};

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pEvent {
    WebRTC(P2pWebRTCEvent),
    LibP2p(P2pLibP2pEvent),
    Channel(P2pChannelEvent),
    #[cfg(not(target_arch = "wasm32"))]
    Libp2pIdentify(PeerId, libp2p::Multiaddr),
    Discovery(P2pDiscoveryEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pWebRTCEvent {
    OfferSdpReady(PeerId, Result<String, String>),
    AnswerSdpReady(PeerId, Result<String, String>),
    AnswerReceived(PeerId, P2pConnectionWebRTCResponse),
    Finalized(PeerId, Result<(), String>),
    Closed(PeerId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pLibP2pEvent {
    IncomingConnection {
        connection_id: P2pConnectionId,
    },
    Dialing {
        connection_id: P2pConnectionId,
    },
    IncomingConnectionError {
        connection_id: P2pConnectionId,
        error: String,
    },
    OutgoingConnectionError {
        connection_id: P2pConnectionId,
        peer_id: PeerId,
        error: String,
    },
    ConnectionEstablished {
        peer_id: PeerId,
        connection_id: P2pConnectionId,
    },
    ConnectionClosed {
        peer_id: PeerId,
        cause: Option<String>,
    },
    NewListenAddr {
        listener_id: P2pListenerId,
        addr: libp2p::Multiaddr,
    },
    ExpiredListenAddr {
        listener_id: P2pListenerId,
        addr: libp2p::Multiaddr,
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
    AddRoute(PeerId, P2pGenericAddrs),
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
            Self::WebRTC(v) => v.fmt(f),
            Self::LibP2p(v) => v.fmt(f),
            Self::Channel(v) => v.fmt(f),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Libp2pIdentify(peer_id, addr) => {
                write!(f, "{peer_id} {addr}")
            }
            Self::Discovery(v) => v.fmt(f),
        }
    }
}

impl fmt::Display for P2pLibP2pEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LibP2p, ")?;
        match self {
            P2pLibP2pEvent::NewListenAddr { listener_id, addr } => {
                write!(f, "NewListenAddr, {listener_id}, {addr}")
            }
            P2pLibP2pEvent::ExpiredListenAddr { listener_id, addr } => {
                write!(f, "ExpiredListenAddr, {listener_id}, {addr}")
            }
            P2pLibP2pEvent::ListenerError { listener_id, error } => {
                write!(f, "ListenerError, {listener_id}, {error}")
            }
            P2pLibP2pEvent::ListenerClosed {
                listener_id,
                error: Some(error),
            } => write!(f, "ListenerClosed, {listener_id}, {error}"),
            P2pLibP2pEvent::ListenerClosed {
                listener_id,
                error: None,
            } => write!(f, "ListenerClosed, {listener_id}"),
            P2pLibP2pEvent::IncomingConnection { connection_id } => {
                write!(f, "IncomingConnection, {connection_id}")
            }
            P2pLibP2pEvent::IncomingConnectionError {
                connection_id,
                error,
            } => write!(f, "IncomingConnectionError, {connection_id}, {error}"),
            P2pLibP2pEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => write!(
                f,
                "OutgoingConnectionError, {connection_id}, {peer_id}, {error}"
            ),
            P2pLibP2pEvent::Dialing { connection_id } => write!(f, "Dialing, {connection_id}"),
            P2pLibP2pEvent::ConnectionEstablished {
                peer_id,
                connection_id,
            } => write!(f, "ConnectionEstablished, {peer_id}, {connection_id}"),
            P2pLibP2pEvent::ConnectionClosed { peer_id, cause } => {
                write!(f, "ConnectionClosed, {peer_id}, {cause:?}")
            }
        }
    }
}

impl fmt::Display for P2pWebRTCEvent {
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
                P2pConnectionWebRTCResponse::Accepted(_) => {
                    write!(f, "AnswerReceived, {peer_id}, Accepted")
                }
                P2pConnectionWebRTCResponse::Rejected(reason) => {
                    write!(f, "AnswerReceived, {peer_id}, Rejected, {reason:?}")
                }
                P2pConnectionWebRTCResponse::InternalError => {
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
            Self::AddRoute(peer_id, opts) => write!(f, "AddRoute, peer_id: {peer_id}, {opts}",),
        }
    }
}

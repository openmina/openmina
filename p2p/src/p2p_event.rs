use derive_more::From;
use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use libp2p::Multiaddr;

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId},
    connection::P2pConnectionResponse,
    PeerId,
};

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pEvent {
    Connection(P2pConnectionEvent),
    Channel(P2pChannelEvent),
    Libp2pIdentify(PeerId, Multiaddr),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionEvent {
    OfferSdpReady(PeerId, Result<String, String>),
    AnswerSdpReady(PeerId, Result<String, String>),
    AnswerReceived(PeerId, P2pConnectionResponse),
    Finalized(PeerId, Result<(), String>),
    Closed(PeerId),
}

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pChannelEvent {
    Opened(PeerId, ChannelId, Result<(), String>),
    Sent(PeerId, ChannelId, MsgId, Result<(), String>),
    Received(PeerId, Result<ChannelMsg, String>),
    Libp2pSnarkReceived(PeerId, Snark, u32),
    Closed(PeerId, ChannelId),
}

fn res_kind<T, E>(res: &Result<T, E>) -> &'static str {
    match res {
        Err(_) => "Err",
        Ok(_) => "Ok",
    }
}

impl std::fmt::Display for P2pEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P2p, ")?;
        match self {
            Self::Connection(v) => v.fmt(f),
            Self::Channel(v) => v.fmt(f),
            Self::Libp2pIdentify(peer_id, addr) => {
                write!(f, "{peer_id} {addr}")
            }
        }
    }
}

impl std::fmt::Display for P2pConnectionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for P2pChannelEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

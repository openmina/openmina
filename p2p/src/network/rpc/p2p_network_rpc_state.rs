use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    str,
    time::Duration,
};

use binprot::BinProtWrite;
use serde::{Deserialize, Serialize};

use mina_p2p_messages::{
    rpc_kernel::{MessageHeader, QueryHeader, ResponseHeader},
    string::CharString,
    versioned::Ver,
};

use crate::{channels::rpc::P2pRpcId, Data};

use super::super::*;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcState {
    pub addr: SocketAddr,
    pub stream_id: StreamId,
    pub last_id: P2pRpcId,
    pub last_heartbeat_sent: Option<redux::Timestamp>,
    pub pending: Option<QueryHeader>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub total_stats: BTreeMap<(CharString, Ver), usize>,
    pub is_incoming: bool,
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<RpcMessage>,
    pub error: Option<P2pNetworkRpcError>,
}

impl P2pNetworkRpcState {
    pub fn new(addr: SocketAddr, stream_id: StreamId) -> Self {
        P2pNetworkRpcState {
            addr,
            stream_id,
            last_id: 0,
            last_heartbeat_sent: None,
            pending: None,
            total_stats: BTreeMap::default(),
            is_incoming: false,
            buffer: vec![],
            incoming: Default::default(),
            error: None,
        }
    }

    pub fn should_send_heartbeat(&self, now: redux::Timestamp) -> bool {
        self.last_heartbeat_sent.map_or(true, |last_sent| {
            now.checked_sub(last_sent)
                .map_or(false, |dur| dur >= HEARTBEAT_INTERVAL)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcMessage {
    Handshake,
    Heartbeat,
    Query { header: QueryHeader, bytes: Data },
    Response { header: ResponseHeader, bytes: Data },
}

const HANDSHAKE_ID: P2pRpcId = P2pRpcId::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00");

impl RpcMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 8];
        match self {
            Self::Handshake => {
                MessageHeader::Response(ResponseHeader { id: HANDSHAKE_ID })
                    .binprot_write(&mut v)
                    .unwrap_or_default();
                v.extend_from_slice(b"\x01");
            }
            Self::Heartbeat => {
                MessageHeader::Heartbeat
                    .binprot_write(&mut v)
                    .unwrap_or_default();
            }
            Self::Query { header, bytes } => {
                MessageHeader::Query(header.clone())
                    .binprot_write(&mut v)
                    .unwrap_or_default();
                v.extend_from_slice(&bytes);
            }
            Self::Response { header, bytes } => {
                MessageHeader::Response(header)
                    .binprot_write(&mut v)
                    .unwrap_or_default();
                v.extend_from_slice(&bytes);
            }
        }

        let len_bytes = ((v.len() - 8) as u64).to_le_bytes();
        v[..8].clone_from_slice(&len_bytes);
        v
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkRpcError {
    #[error("error reading binprot message: {0}")]
    Binprot(String),
    #[error("message {0} with size {1} exceeds limit of {2}")]
    Limit(String, usize, Limit<usize>),
}

use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    str,
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

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcState {
    pub addr: SocketAddr,
    pub stream_id: StreamId,
    pub last_id: P2pRpcId,
    pub pending: Option<(P2pRpcId, (CharString, Ver))>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub total_stats: BTreeMap<(CharString, Ver), usize>,
    pub is_incoming: bool,
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<RpcMessage>,
    pub error: Option<String>,
}

impl P2pNetworkRpcState {
    pub fn new(addr: SocketAddr, stream_id: StreamId) -> Self {
        P2pNetworkRpcState {
            addr,
            stream_id,
            last_id: 0,
            pending: None,
            total_stats: BTreeMap::default(),
            is_incoming: false,
            buffer: vec![],
            incoming: Default::default(),
            error: None,
        }
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
                MessageHeader::Query(header)
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

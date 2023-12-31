use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    str,
};

use binprot::{BinProtRead, BinProtWrite};
use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use mina_p2p_messages::{
    rpc,
    rpc_kernel::{
        Error as RpcError, MessageHeader, NeedsLength, QueryHeader, QueryPayload, ResponseHeader,
        ResponsePayload, RpcMethod,
    },
    string::CharString,
};

use crate::{
    channels::rpc::{BestTipWithProof, P2pChannelsRpcAction, P2pRpcResponse},
    Data,
};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcState {
    pub addr: SocketAddr,
    pub stream_id: StreamId,
    pub last_id: i64,
    pub pending: BTreeMap<i64, (CharString, i32)>,
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
            pending: BTreeMap::default(),
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

const HANDSHAKE_ID: i64 = i64::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00");

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

impl P2pNetworkRpcState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkRpcAction>) {
        if self.error.is_some() {
            return;
        }
        match action.action() {
            P2pNetworkRpcAction::Init(a) => {
                self.is_incoming = a.incoming;
            }
            P2pNetworkRpcAction::IncomingData(a) => {
                self.buffer.extend_from_slice(&a.data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if buf.len() >= 8 {
                        let len = u64::from_le_bytes(
                            buf[..8].try_into().expect("cannot fail, checked above"),
                        ) as usize;
                        if buf.len() >= 8 + len {
                            offset += 8 + len;
                            let mut slice = &buf[8..(8 + len)];
                            let msg = match MessageHeader::binprot_read(&mut slice) {
                                Ok(MessageHeader::Heartbeat) => RpcMessage::Heartbeat,
                                Ok(MessageHeader::Response(h))
                                    if h.id == i64::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00") =>
                                {
                                    RpcMessage::Handshake
                                }
                                Ok(MessageHeader::Query(header)) => RpcMessage::Query {
                                    header,
                                    bytes: slice.to_vec().into(),
                                },
                                Ok(MessageHeader::Response(header)) => RpcMessage::Response {
                                    header,
                                    bytes: slice.to_vec().into(),
                                },
                                Err(err) => {
                                    self.error = Some(err.to_string());
                                    continue;
                                }
                            };
                            self.incoming.push_back(msg);
                            continue;
                        }
                    }

                    break;
                }

                self.buffer = self.buffer[offset..].to_vec();
            }
            P2pNetworkRpcAction::IncomingMessage(_) => {
                self.incoming.pop_front();
            }
            P2pNetworkRpcAction::OutgoingQuery(a) => {
                self.last_id = a.query.id;
                // TODO: remove when query is done
                self.pending
                    .insert(a.query.id, (a.query.tag.clone(), a.query.version));
            }
            _ => {}
        }
    }
}

impl P2pNetworkRpcAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pNetworkRpcOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingMessageAction: redux::EnablingCondition<S>,
        P2pNetworkRpcOutgoingQueryAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingDataAction: redux::EnablingCondition<S>,
        P2pChannelsRpcAction: redux::EnablingCondition<S>,
    {
        let Some(state) = store.state().network.find_rpc_state(self) else {
            return;
        };

        let incoming = state.incoming.front().cloned();

        match self {
            Self::Init(a) => {
                store.dispatch(P2pNetworkRpcOutgoingDataAction {
                    addr: a.addr,
                    peer_id: a.peer_id,
                    stream_id: a.stream_id,
                    data: RpcMessage::Handshake.into_bytes().into(),
                    fin: false,
                });
            }
            Self::IncomingData(a) => {
                if let Some(message) = incoming {
                    store.dispatch(P2pNetworkRpcIncomingMessageAction {
                        addr: a.addr,
                        peer_id: a.peer_id,
                        stream_id: a.stream_id,
                        message,
                    });
                }
            }
            Self::IncomingMessage(a) => {
                // TODO: process
                match &a.message {
                    RpcMessage::Handshake => {
                        if !state.is_incoming {
                            let mut v = vec![];

                            // type Payload =
                            //     QueryPayload<<rpc::VersionedRpcMenuV1 as RpcMethod>::Query>;
                            // <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                            //     .unwrap_or_default();

                            // store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                            //     peer_id: a.peer_id,
                            //     query: QueryHeader {
                            //         tag: rpc::VersionedRpcMenuV1::NAME.into(),
                            //         version: rpc::VersionedRpcMenuV1::VERSION,
                            //         id: state.last_id,
                            //     },
                            //     data: v.into(),
                            // });

                            type Payload = QueryPayload<<rpc::GetBestTipV2 as RpcMethod>::Query>;
                            <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                                .unwrap_or_default();

                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id: a.peer_id,
                                query: QueryHeader {
                                    tag: rpc::GetBestTipV2::NAME.into(),
                                    version: rpc::GetBestTipV2::VERSION,
                                    id: state.last_id,
                                },
                                data: v.into(),
                            });
                        }
                    }
                    RpcMessage::Heartbeat => {
                        store.dispatch(P2pNetworkRpcOutgoingDataAction {
                            addr: a.addr,
                            peer_id: a.peer_id,
                            stream_id: a.stream_id,
                            data: RpcMessage::Heartbeat.into_bytes().into(),
                            fin: false,
                        });
                    }
                    RpcMessage::Query { .. } => {
                        // TODO: dispatch further action
                    }
                    RpcMessage::Response { header, bytes } => {
                        fn parse_r<M: RpcMethod>(
                            bytes: &[u8],
                        ) -> Result<Result<M::Response, RpcError>, String> {
                            let mut bytes = bytes;
                            <ResponsePayload<M::Response> as BinProtRead>::binprot_read(&mut bytes)
                                .map(|x| x.0.map(|NeedsLength(x)| x))
                                .map_err(|err| format!("response {} {}", M::NAME, err))
                        }

                        if let Some((tag, version)) = state.pending.get(&header.id) {
                            if let Ok(tag) = std::str::from_utf8(tag.as_ref()) {
                                match (tag, *version) {
                                    (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
                                        let Ok(response) = parse_r::<rpc::GetBestTipV2>(&bytes)
                                        else {
                                            // TODO: close the stream
                                            return;
                                        };
                                        let response = response
                                            .ok()
                                            .flatten()
                                            .map(|resp| BestTipWithProof {
                                                best_tip: resp.data.into(),
                                                proof: (resp.proof.0, resp.proof.1.into()),
                                            })
                                            .map(P2pRpcResponse::BestTipWithProof);

                                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                                            peer_id: a.peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::AnswerSyncLedgerQueryV2::NAME,
                                        rpc::AnswerSyncLedgerQueryV2::VERSION,
                                    ) => {
                                        let Ok(response) =
                                            parse_r::<rpc::AnswerSyncLedgerQueryV2>(&bytes)
                                        else {
                                            // TODO: close the stream
                                            return;
                                        };

                                        let response = response
                                            .ok()
                                            .map(|x| x.0.ok())
                                            .flatten()
                                            .map(P2pRpcResponse::LedgerQuery);

                                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                                            peer_id: a.peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // TODO: dispatch further action
                    }
                }

                if let Some(message) = incoming {
                    store.dispatch(P2pNetworkRpcIncomingMessageAction {
                        addr: a.addr,
                        peer_id: a.peer_id,
                        stream_id: a.stream_id,
                        message,
                    });
                }
            }
            Self::OutgoingQuery(a) => {
                store.dispatch(P2pNetworkRpcOutgoingDataAction {
                    addr: state.addr,
                    peer_id: a.peer_id,
                    stream_id: state.stream_id,
                    data: RpcMessage::Query {
                        header: a.query.clone(),
                        bytes: a.data.clone(),
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            Self::OutgoingData(a) => {
                store.dispatch(P2pNetworkYamuxOutgoingDataAction {
                    addr: a.addr,
                    stream_id: a.stream_id,
                    data: a.data.clone(),
                    fin: false,
                });
            }
        }
    }
}

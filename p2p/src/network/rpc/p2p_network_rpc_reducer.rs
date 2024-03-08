use binprot::BinProtRead;
use mina_p2p_messages::rpc_kernel::MessageHeader;

use crate::channels::rpc::P2pChannelsRpcState;

use super::*;

impl P2pNetworkRpcState {
    pub fn reducer(
        &mut self,
        rpc_state: &mut P2pChannelsRpcState,
        action: redux::ActionWithMeta<&P2pNetworkRpcAction>,
    ) {
        if self.error.is_some() {
            return;
        }
        match action.action() {
            P2pNetworkRpcAction::Init(a) => {
                self.is_incoming = a.incoming;
                *rpc_state = P2pChannelsRpcState::Pending {
                    time: action.time(),
                };
            }
            P2pNetworkRpcAction::IncomingData(a) => {
                self.buffer.extend_from_slice(&a.data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if let Some(len_bytes) = buf.get(..8).and_then(|s| s.try_into().ok()) {
                        let len = u64::from_le_bytes(len_bytes) as usize;
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
                                Ok(MessageHeader::Response(header)) => {
                                    if let Some((_, req)) = &self.pending {
                                        *self.total_stats.entry(req.clone()).or_default() += 1;
                                    } else {
                                        // suspisious, peer sent us response, but no request
                                    }
                                    RpcMessage::Response {
                                        header,
                                        bytes: slice.to_vec().into(),
                                    }
                                }
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

                if offset != 0 {
                    self.buffer = self.buffer[offset..].to_vec();
                }
            }
            P2pNetworkRpcAction::IncomingMessage(a) => {
                if matches!(&a.message, RpcMessage::Response { .. }) {
                    if let Some((_, req)) = &self.pending {
                        *self.total_stats.entry(req.clone()).or_default() += 1;
                    } else {
                        // suspicious, received some response without request
                    }
                }
                self.incoming.pop_front();
            }
            P2pNetworkRpcAction::OutgoingQuery(a) => {
                self.last_id = a.query.id;
                // TODO: remove when query is done
                self.pending = Some((a.query.id, (a.query.tag.clone(), a.query.version)));
            }
            _ => {}
        }
    }
}

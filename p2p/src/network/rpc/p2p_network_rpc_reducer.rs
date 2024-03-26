use binprot::BinProtRead;
use mina_p2p_messages::rpc_kernel::MessageHeader;

use super::*;

impl P2pNetworkRpcState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkRpcAction>) {
        if self.error.is_some() {
            return;
        }
        match action.action() {
            P2pNetworkRpcAction::Init { incoming, .. } => {
                self.is_incoming = *incoming;
            }
            P2pNetworkRpcAction::IncomingData { data, .. } => {
                self.buffer.extend_from_slice(&data);
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

                if offset != 0 {
                    self.buffer = self.buffer[offset..].to_vec();
                }
            }
            P2pNetworkRpcAction::IncomingMessage { message, .. } => {
                if matches!(&message, RpcMessage::Response { .. }) {
                    if let Some((_, req)) = &self.pending {
                        *self.total_stats.entry(req.clone()).or_default() += 1;
                    } else {
                        // suspicious, received some response without request
                    }
                }
                self.incoming.pop_front();
            }
            P2pNetworkRpcAction::OutgoingQuery { query, .. } => {
                self.last_id = query.id;
                // TODO: remove when query is done
                self.pending = Some((query.id, (query.tag.clone(), query.version)));
            }
            _ => {}
        }
    }
}

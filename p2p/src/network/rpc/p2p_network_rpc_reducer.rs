use binprot::BinProtRead;
use mina_p2p_messages::rpc_kernel::{MessageHeader, QueryHeader, RpcMethod};

use crate::{Limit, P2pLimits};

use self::p2p_network_rpc_state::P2pNetworkRpcError;

use super::*;

impl P2pNetworkRpcState {
    pub fn reducer(
        &mut self,
        action: redux::ActionWithMeta<&P2pNetworkRpcAction>,
        limits: &P2pLimits,
    ) {
        if self.error.is_some() {
            return;
        }
        match action.action() {
            P2pNetworkRpcAction::Init { incoming, .. } => {
                self.is_incoming = *incoming;
            }
            P2pNetworkRpcAction::IncomingData { data, .. } => {
                self.buffer.extend_from_slice(data);
                let mut offset = 0;
                // TODO(akoptelov): there shouldn't be the case where we have multiple incoming messages at once (or at least other than heartbeat)
                loop {
                    let buf = &self.buffer[offset..];
                    if let Some(len_bytes) = buf.get(..8).and_then(|s| s.try_into().ok()) {
                        let len = u64::from_le_bytes(len_bytes) as usize;
                        if let Err(err) = self.check_rpc_limit(len, limits) {
                            self.error = Some(err);
                            return;
                        }
                        if buf.len() >= 8 + len {
                            offset += 8 + len;
                            let mut slice = &buf[8..(8 + len)];
                            let msg = match MessageHeader::binprot_read(&mut slice) {
                                Ok(MessageHeader::Heartbeat) => RpcMessage::Heartbeat,
                                Ok(MessageHeader::Response(h))
                                    if h.id == u64::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00") =>
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
                                    self.error = Some(P2pNetworkRpcError::Binprot(err.to_string()));
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
                if let RpcMessage::Response { header, .. } = message {
                    if let Some(QueryHeader { id, tag, version }) = &self.pending {
                        *self.total_stats.entry((tag.clone(), *version)).or_default() += 1;
                        if id != &header.id {
                            openmina_core::error!(action.time(); "receiving response with wrong id: {}", header.id);
                        }
                    } else {
                        openmina_core::error!(action.time(); "receiving response without query");
                    }
                } else if let RpcMessage::Query { header, .. } = message {
                    if self.pending.is_none() {
                        self.pending = Some(header.clone());
                    } else {
                        openmina_core::error!(action.time(); "receiving query while another query is pending");
                    }
                }

                self.incoming.pop_front();
            }
            P2pNetworkRpcAction::PrunePending { .. } => {
                self.pending = None;
            }
            P2pNetworkRpcAction::OutgoingQuery { query, .. } => {
                self.last_id = query.id;
                self.pending = Some(query.clone());
            }
            _ => {}
        }
    }

    fn check_rpc_limit(&self, len: usize, limits: &P2pLimits) -> Result<(), P2pNetworkRpcError> {
        let (limit, kind): (_, &[u8]) = if self.is_incoming {
            // only requests are allowed
            (limits.rpc_query(), b"<query>")
        } else if let Some(QueryHeader { tag, .. }) = self.pending.as_ref() {
            use mina_p2p_messages::rpc::*;
            match tag.as_ref() {
                GetBestTipV2::NAME => (limits.rpc_get_best_tip(), GetBestTipV2::NAME),
                AnswerSyncLedgerQueryV2::NAME => (
                    limits.rpc_answer_sync_ledger_query(),
                    AnswerSyncLedgerQueryV2::NAME,
                ),
                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME => (
                    limits.rpc_get_staged_ledger(),
                    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                ),
                GetTransitionChainV2::NAME => (
                    limits.rpc_get_transition_chain(),
                    GetTransitionChainV2::NAME,
                ),
                GetSomeInitialPeersV1ForV2::NAME => (
                    limits.rpc_get_some_initial_peers(),
                    GetSomeInitialPeersV1ForV2::NAME,
                ),
                _ => (Limit::Some(0), b"<unimplemented>"),
            }
        } else {
            (limits.rpc_service_message(), b"<service_messages>")
        };
        let kind = String::from_utf8_lossy(kind);
        if len > limit {
            Err(P2pNetworkRpcError::Limit(kind.into_owned(), len, limit))
        } else {
            Ok(())
        }
    }
}

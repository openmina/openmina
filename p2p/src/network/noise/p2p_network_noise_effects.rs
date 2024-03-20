use super::{super::*, *};

use super::p2p_network_noise_state::{
    P2pNetworkNoiseStateInitiator, P2pNetworkNoiseStateResponder,
};

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let state = store.state();
        let Some(state) = state.network.scheduler.connections.get(&self.addr()) else {
            return;
        };
        let Some(P2pNetworkAuthState::Noise(state)) = &state.auth else {
            return;
        };

        let incoming = state.incoming_chunks.front().cloned().map(Into::into);
        let outgoing = state.outgoing_chunks.front().cloned().map(Into::into);
        let decrypted = state.decrypted_chunks.front().cloned();
        let remote_peer_id = match &state.inner {
            Some(P2pNetworkNoiseStateInner::Done { remote_peer_id, .. }) => {
                Some(remote_peer_id.clone())
            }
            Some(P2pNetworkNoiseStateInner::Initiator(P2pNetworkNoiseStateInitiator {
                remote_pk: Some(pk),
                ..
            })) => Some(pk.peer_id()),
            _ => None,
        };
        let handshake_done = if let Some(P2pNetworkNoiseStateInner::Done {
            remote_peer_id,
            incoming,
            send_nonce,
            recv_nonce,
            ..
        }) = &state.inner
        {
            if ((matches!(self, Self::IncomingChunk(..)) && *incoming)
                || (matches!(self, Self::OutgoingChunk(..)) && !*incoming))
                && *send_nonce == 0
                && *recv_nonce == 0
            {
                Some((remote_peer_id.clone(), *incoming))
            } else {
                None
            }
        } else {
            None
        };
        let handshake_optimized = state.handshake_optimized;
        let middle_initiator =
            matches!(&state.inner, Some(P2pNetworkNoiseStateInner::Initiator(..)))
                && remote_peer_id.is_some();
        let middle_responder = matches!(
            &state.inner,
            Some(P2pNetworkNoiseStateInner::Responder(
                P2pNetworkNoiseStateResponder::Init { .. },
            ))
        );

        if let Self::HandshakeDone(a) = self {
            store.dispatch(P2pNetworkSelectInitAction {
                addr: a.addr,
                kind: SelectKind::Multiplexing(a.peer_id.clone()),
                incoming: a.incoming,
                send_handshake: true,
            });
            return;
        }

        if let Self::DecryptedData(a) = self {
            let kind = match &a.peer_id.or(remote_peer_id) {
                Some(peer_id) => SelectKind::Multiplexing(peer_id.clone()),
                None => SelectKind::MultiplexingNoPeerId,
            };
            if handshake_optimized && middle_initiator {
                store.dispatch(P2pNetworkSelectInitAction {
                    addr: self.addr(),
                    kind,
                    // it is not a mistake, if we are initiator of noise, the select will be incoming
                    // because noise is
                    // initiator -> responder (ephemeral key)
                    // initiator <- responder (ephemeral key, encrypted static kay and **encrypted payload**)
                    // initiator -> responder (encrypted static kay and **encrypted payload**)
                    // so the responder is sending payload first, hence responder will be initiator of underlying protocol
                    incoming: true,
                    send_handshake: false,
                });
            }
            store.dispatch(P2pNetworkSelectIncomingDataAction {
                addr: self.addr(),
                kind,
                data: a.data.clone(),
                fin: false,
            });
            return;
        }

        match self {
            Self::Init(_) | Self::OutgoingData(_) => {
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::IncomingData(_) => {
                if let Some(data) = incoming {
                    store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::IncomingChunk(_) => {
                if handshake_optimized && middle_responder {
                    let kind = match &remote_peer_id {
                        Some(peer_id) => SelectKind::Multiplexing(peer_id.clone()),
                        None => SelectKind::MultiplexingNoPeerId,
                    };

                    store.dispatch(P2pNetworkSelectInitAction {
                        addr: self.addr(),
                        kind,
                        incoming: false,
                        send_handshake: false,
                    });
                }

                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
                if let Some(data) = decrypted {
                    store.dispatch(P2pNetworkNoiseDecryptedDataAction {
                        addr: self.addr(),
                        peer_id: remote_peer_id,
                        data,
                    });
                }
                if let Some(data) = incoming {
                    store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::OutgoingChunk(a) => {
                store.dispatch(P2pNetworkPnetOutgoingDataAction {
                    addr: a.addr,
                    data: a.data.clone(),
                });
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            _ => {}
        }

        if !handshake_optimized {
            if (middle_initiator || middle_responder) && matches!(self, Self::IncomingChunk(..)) {
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: self.addr(),
                    data: Data(vec![].into_boxed_slice()),
                });
            } else {
                if let Some((peer_id, incoming)) = handshake_done {
                    store.dispatch(P2pNetworkNoiseHandshakeDoneAction {
                        addr: self.addr(),
                        peer_id,
                        incoming,
                    });
                }
            }
        }
    }
}

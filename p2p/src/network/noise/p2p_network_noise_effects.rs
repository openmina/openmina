use crate::connection::incoming::{P2pConnectionIncomingAction, P2pConnectionIncomingState};
use crate::FUZZ;

use super::{super::*, *};

use super::p2p_network_noise_state::{
    P2pNetworkNoiseStateInitiator, P2pNetworkNoiseStateResponder,
};

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let state = store.state();
        let Some(state) = state.network.scheduler.connections.get(self.addr()) else {
            return;
        };
        let Some(P2pNetworkAuthState::Noise(state)) = &state.auth else {
            return;
        };

        let incoming = state.incoming_chunks.clone();
        let outgoing = state.outgoing_chunks.front().cloned();
        let decrypted = state.decrypted_chunks.front().cloned();
        let remote_peer_id = match &state.inner {
            Some(P2pNetworkNoiseStateInner::Done { remote_peer_id, .. }) => Some(*remote_peer_id),
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
            if ((matches!(self, Self::IncomingChunk { .. }) && *incoming)
                || (matches!(self, Self::OutgoingChunk { .. }) && !*incoming))
                && *send_nonce == 0
                && *recv_nonce == 0
            {
                Some((*remote_peer_id, *incoming))
            } else {
                None
            }
        } else {
            None
        };
        let handshake_error = if let Some(P2pNetworkNoiseStateInner::Error(error)) = &state.inner {
            Some(error)
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

        if let Self::HandshakeDone {
            addr,
            peer_id,
            incoming,
        } = self
        {
            store.dispatch(P2pNetworkSelectAction::Init {
                addr,
                kind: SelectKind::Multiplexing(peer_id),
                incoming,
                send_handshake: true,
            });
            return;
        }

        if let Self::DecryptedData {
            addr,
            peer_id,
            data,
        } = self
        {
            let kind = match &peer_id.or(remote_peer_id) {
                Some(peer_id) => SelectKind::Multiplexing(*peer_id),
                None => SelectKind::MultiplexingNoPeerId,
            };
            if handshake_optimized && middle_initiator {
                store.dispatch(P2pNetworkSelectAction::Init {
                    addr,
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
            store.dispatch(P2pNetworkSelectAction::IncomingData {
                addr,
                kind,
                data: data.clone(),
                fin: false,
            });
            return;
        }

        match self {
            Self::Init { addr, .. } | Self::OutgoingData { addr, .. } => {
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseAction::OutgoingChunk { addr, data });
                }
            }
            Self::IncomingData { addr, .. } => {
                let mut incoming = incoming;
                while let Some(data) = incoming.pop_front() {
                    store.dispatch(P2pNetworkNoiseAction::IncomingChunk {
                        addr,
                        data: data.into(),
                    });
                }
            }
            Self::IncomingChunk { addr, .. } => {
                if let Some(error) = handshake_error {
                    store.dispatch(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: error.clone().into(),
                    });
                    return;
                }

                if let Some((peer_id, true)) = handshake_done {
                    let addr = *self.addr();
                    store.dispatch(P2pConnectionIncomingAction::FinalizePendingLibp2p {
                        peer_id,
                        addr,
                    });
                    // check that peer management decide to accept this connection
                    let this_connection_is_kept = store
                        .state()
                        .peers
                        .get(&peer_id)
                        .and_then(|peer_state| peer_state.status.as_connecting())
                        .and_then(|connecting| connecting.as_incoming())
                        .map_or(false, |incoming| matches!(incoming, P2pConnectionIncomingState::FinalizePendingLibp2p { addr: a, .. } if a == &addr));
                    if !this_connection_is_kept {
                        return;
                    }
                }

                if handshake_optimized && middle_responder {
                    let kind = match &remote_peer_id {
                        Some(peer_id) => SelectKind::Multiplexing(*peer_id),
                        None => SelectKind::MultiplexingNoPeerId,
                    };

                    store.dispatch(P2pNetworkSelectAction::Init {
                        addr,
                        kind,
                        incoming: false,
                        send_handshake: false,
                    });
                }

                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseAction::OutgoingChunk { addr, data });
                }
                if let Some(data) = decrypted {
                    store.dispatch(P2pNetworkNoiseAction::DecryptedData {
                        addr,
                        peer_id: remote_peer_id,
                        data,
                    });
                }

                if !handshake_optimized && (middle_initiator || middle_responder) {
                    store.dispatch(P2pNetworkNoiseAction::OutgoingData {
                        addr,
                        data: Data(vec![].into_boxed_slice()),
                    });
                }
            }
            Self::OutgoingChunk { addr, data } => {
                let mut data = data
                    .iter()
                    .fold(vec![], |mut v, item| {
                        v.extend_from_slice(item);
                        v
                    })
                    .into();

                if let Ok(mut fuzzer) = FUZZ.lock() {
                    fuzzer
                        .as_mut()
                        .map(|fuzzer| fuzzer.mutate_noise(&mut data));
                }

                store.dispatch(P2pNetworkPnetAction::OutgoingData { addr, data });
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseAction::OutgoingChunk { addr, data });
                }
                if let Some((peer_id, incoming)) = handshake_done {
                    store.dispatch(P2pNetworkNoiseAction::HandshakeDone {
                        addr,
                        peer_id,
                        incoming,
                    });
                }
            }
            _ => {}
        }
    }
}

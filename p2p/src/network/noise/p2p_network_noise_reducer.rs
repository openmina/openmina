use chacha20poly1305::{aead::generic_array::GenericArray, AeadInPlace, ChaCha20Poly1305, KeyInit};
use crypto_bigint::consts::U12;
use openmina_core::{bug_condition, fuzzed_maybe, Substate};

use crate::connection::incoming::{P2pConnectionIncomingAction, P2pConnectionIncomingState};
use crate::{
    Data, P2pNetworkConnectionError, P2pNetworkPnetAction, P2pNetworkSchedulerAction,
    P2pNetworkSchedulerState, P2pNetworkSelectAction, P2pState, SelectKind,
};

use self::p2p_network_noise_state::ResponderConsumeOutput;

use super::*;

use super::p2p_network_noise_state::{
    InitiatorOutput, NoiseError, NoiseState, P2pNetworkNoiseState, P2pNetworkNoiseStateInitiator,
    P2pNetworkNoiseStateInner, P2pNetworkNoiseStateResponder, ResponderOutput,
};

const MAX_CHUNK_SIZE: usize = u16::MAX as usize - 19;

impl P2pNetworkNoiseState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkSchedulerState>,
        action: redux::ActionWithMeta<&P2pNetworkNoiseAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let action = action.action();
        let noise_state = state_context
            .get_substate_mut()?
            .connection_state_mut(action.addr())
            .and_then(|c| c.noise_state_mut())
            .ok_or_else(|| "Invalid noise state".to_owned())?;

        match action {
            P2pNetworkNoiseAction::Init {
                incoming,
                ephemeral_sk,
                static_sk,
                signature,
                addr,
            } => {
                let esk = ephemeral_sk.clone();
                let epk = esk.pk();
                let ssk = static_sk.clone();
                let spk = ssk.pk();
                let payload = signature.clone();

                noise_state.inner = if *incoming {
                    // Luckily the name is 32 bytes long, if it were longer you would have to take a sha2_256 hash of it.
                    let mut noise = NoiseState::new(*b"Noise_XX_25519_ChaChaPoly_SHA256");
                    noise.mix_hash(b"");

                    Some(P2pNetworkNoiseStateInner::Responder(
                        P2pNetworkNoiseStateResponder::Init {
                            r_esk: esk,
                            r_spk: spk,
                            r_ssk: ssk,
                            buffer: vec![],
                            payload,
                            noise,
                        },
                    ))
                } else {
                    let mut chunk = vec![0, 32];
                    chunk.extend_from_slice(epk.0.as_bytes());
                    noise_state.outgoing_chunks.push_back(vec![chunk.into()]);

                    let mut noise = NoiseState::new(*b"Noise_XX_25519_ChaChaPoly_SHA256");
                    noise.mix_hash(b"");
                    noise.mix_hash(epk.0.as_bytes());
                    noise.mix_hash(b"");

                    Some(P2pNetworkNoiseStateInner::Initiator(
                        P2pNetworkNoiseStateInitiator {
                            i_esk: esk,
                            i_spk: spk,
                            i_ssk: ssk,
                            r_epk: None,
                            payload,
                            noise,
                            remote_pk: None,
                        },
                    ))
                };

                let mut outgoing = noise_state.outgoing_chunks.clone();
                let dispatcher = state_context.into_dispatcher();
                while let Some(data) = outgoing.pop_front() {
                    dispatcher.push(P2pNetworkNoiseAction::OutgoingChunk { addr: *addr, data });
                }

                Ok(())
            }
            P2pNetworkNoiseAction::IncomingData { data, addr } => {
                noise_state.buffer.extend_from_slice(data);
                let mut offset = 0;
                loop {
                    let buf = &noise_state.buffer[offset..];
                    // Note: there is no way to determine if a peer is not sending more data on purpose or not.
                    // Let the timeout logic handle this.
                    let len = buf
                        .get(..2)
                        .and_then(|buf| Some(u16::from_be_bytes(buf.try_into().ok()?)));

                    if let Some(len) = len {
                        let full_len = 2 + len as usize;
                        if buf.len() >= full_len {
                            noise_state
                                .incoming_chunks
                                .push_back(buf[..full_len].to_vec());
                            offset += full_len;
                            continue;
                        }
                    }
                    break;
                }
                noise_state.buffer = noise_state.buffer[offset..].to_vec();

                let incoming_chunks = noise_state.incoming_chunks.len();
                let dispatcher = state_context.into_dispatcher();
                for _ in 0..incoming_chunks {
                    dispatcher.push(P2pNetworkNoiseAction::IncomingChunk { addr: *addr });
                }
                Ok(())
            }
            action @ P2pNetworkNoiseAction::IncomingChunk { addr } => {
                let Some(state) = &mut noise_state.inner else {
                    bug_condition!("action {:?}: no inner state", action);
                    return Ok(());
                };
                if let Some(mut chunk) = noise_state.incoming_chunks.pop_front() {
                    match state {
                        P2pNetworkNoiseStateInner::Initiator(i) => match i.consume(&mut chunk) {
                            Ok(_) => {}
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(dbg!(err)),
                        },
                        P2pNetworkNoiseStateInner::Responder(o) => match o.consume(&mut chunk) {
                            Ok(None) => {}
                            Ok(Some(ResponderConsumeOutput {
                                output:
                                    ResponderOutput {
                                        send_key,
                                        recv_key,
                                        remote_pk,
                                        ..
                                    },
                                payload: _,
                            })) => {
                                let remote_peer_id = remote_pk.peer_id();

                                if noise_state.expected_peer_id.is_some_and(|expected_per_id| {
                                    expected_per_id != remote_peer_id
                                }) {
                                    *state = P2pNetworkNoiseStateInner::Error(dbg!(
                                        NoiseError::RemotePeerIdMismatch
                                    ));
                                } else {
                                    *state = P2pNetworkNoiseStateInner::Done {
                                        incoming: true,
                                        send_key,
                                        recv_key,
                                        recv_nonce: 0,
                                        send_nonce: 0,
                                        remote_pk,
                                        remote_peer_id,
                                    };
                                }
                            }
                            Err(err) => {
                                *state = P2pNetworkNoiseStateInner::Error(dbg!(err));
                            }
                        },
                        P2pNetworkNoiseStateInner::Done {
                            recv_key,
                            recv_nonce,
                            ..
                        } => {
                            let aead = ChaCha20Poly1305::new(&recv_key.0.into());
                            let mut chunk = chunk;
                            let mut nonce = GenericArray::default();
                            nonce[4..].clone_from_slice(&recv_nonce.to_le_bytes());
                            *recv_nonce += 1;
                            if chunk.len() < 18 {
                                *state = P2pNetworkNoiseStateInner::Error(NoiseError::ChunkTooShort)
                            } else {
                                let data = &mut chunk[2..];
                                let (data, tag) = data.split_at_mut(data.len() - 16);
                                let tag = GenericArray::from_slice(&*tag);
                                if aead
                                    .decrypt_in_place_detached(&nonce, &[], data, tag)
                                    .is_err()
                                {
                                    *state = P2pNetworkNoiseStateInner::Error(dbg!(
                                        NoiseError::FirstMacMismatch
                                    ));
                                } else {
                                    noise_state.decrypted_chunks.push_back(data.to_vec().into());
                                }
                            }
                        }
                        P2pNetworkNoiseStateInner::Error(_) => {}
                    }
                }

                let remote_peer_id = noise_state.remote_peer_id();
                let handshake_done = noise_state.handshake_done(action);
                let mut outgoing = noise_state.outgoing_chunks.clone();
                let decrypted = noise_state.decrypted_chunks.front().cloned();
                let middle_initiator = matches!(
                    &noise_state.inner,
                    Some(P2pNetworkNoiseStateInner::Initiator(..))
                ) && remote_peer_id.is_some();

                let middle_responder = matches!(
                    &noise_state.inner,
                    Some(P2pNetworkNoiseStateInner::Responder(
                        P2pNetworkNoiseStateResponder::Init { .. },
                    ))
                );
                let error = noise_state.as_error();

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                if let Some(error) = error {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr: *addr,
                        error: P2pNetworkConnectionError::Noise(error),
                    });
                } else {
                    if let Some((peer_id, true)) = handshake_done {
                        dispatcher.push(P2pConnectionIncomingAction::FinalizePendingLibp2p {
                            peer_id,
                            addr: addr.sock_addr,
                        });

                        let p2p_state: &P2pState = state.substate()?;
                        let this_connection_is_kept = p2p_state.peers
                        .get(&peer_id)
                        .and_then(|peer_state| peer_state.status.as_connecting())
                        .and_then(|connecting| connecting.as_incoming())
                        .map_or(false, |incoming| matches!(incoming, P2pConnectionIncomingState::FinalizePendingLibp2p { addr: a, .. } if a == &addr.sock_addr));

                        if !this_connection_is_kept {
                            return Ok(());
                        }
                    }

                    while let Some(data) = outgoing.pop_front() {
                        dispatcher.push(P2pNetworkNoiseAction::OutgoingChunk { addr: *addr, data });
                    }

                    if let Some(data) = decrypted {
                        dispatcher.push(P2pNetworkNoiseAction::DecryptedData {
                            addr: *addr,
                            peer_id: remote_peer_id,
                            data,
                        });
                    }

                    if middle_initiator || middle_responder {
                        dispatcher.push(P2pNetworkNoiseAction::OutgoingData {
                            addr: *addr,
                            data: Data::empty(),
                        });
                    }
                }

                Ok(())
            }
            action @ P2pNetworkNoiseAction::OutgoingChunk { addr, data }
            | action @ P2pNetworkNoiseAction::OutgoingChunkSelectMux { addr, data } => {
                noise_state.outgoing_chunks.pop_front();

                let handshake_done = noise_state.handshake_done(action);
                let dispatcher = state_context.into_dispatcher();
                let data = fuzzed_maybe!(
                    data.iter()
                        .fold(vec![], |mut v, item| {
                            v.extend_from_slice(item);
                            v
                        })
                        .into(),
                    crate::fuzzer::mutate_noise
                );
                dispatcher.push(P2pNetworkPnetAction::OutgoingData { addr: *addr, data });
                if let Some((peer_id, incoming)) = handshake_done {
                    dispatcher.push(P2pNetworkNoiseAction::HandshakeDone {
                        addr: *addr,
                        peer_id,
                        incoming,
                    });
                }

                Ok(())
            }
            P2pNetworkNoiseAction::OutgoingData { data, addr } => {
                let Some(state) = &mut noise_state.inner else {
                    bug_condition!("action {:?}: no inner state", action);
                    return Ok(());
                };

                match state {
                    P2pNetworkNoiseStateInner::Done {
                        send_key,
                        send_nonce,
                        ..
                    } => {
                        let aead = ChaCha20Poly1305::new(&send_key.0.into());
                        let mut chunks = vec![];

                        for data_chunk in data.chunks(MAX_CHUNK_SIZE) {
                            let mut chunk = Vec::with_capacity(18 + data_chunk.len());
                            chunk
                                .extend_from_slice(&((data_chunk.len() + 16) as u16).to_be_bytes());
                            chunk.extend_from_slice(data_chunk);

                            let mut nonce: GenericArray<u8, U12> = GenericArray::default();
                            nonce[4..].clone_from_slice(&send_nonce.to_le_bytes());
                            *send_nonce += 1;

                            let tag = aead.encrypt_in_place_detached(
                                &nonce,
                                &[],
                                &mut chunk[2..(2 + data_chunk.len())],
                            );

                            let tag = match tag {
                                Ok(tag) => tag,
                                Err(_) => {
                                    *state =
                                        P2pNetworkNoiseStateInner::Error(NoiseError::Encryption);
                                    return Ok(());
                                }
                            };

                            chunk.extend_from_slice(&tag);
                            chunks.push(chunk.into());
                        }
                        noise_state.outgoing_chunks.push_back(chunks);
                    }
                    P2pNetworkNoiseStateInner::Initiator(i) => {
                        match (i.generate(data), i.remote_pk.clone()) {
                            (
                                Ok(Some(InitiatorOutput {
                                    send_key,
                                    recv_key,
                                    chunk,
                                })),
                                Some(remote_pk),
                            ) => {
                                noise_state.outgoing_chunks.push_back(vec![chunk.into()]);
                                let remote_peer_id = remote_pk.peer_id();

                                if noise_state.expected_peer_id.is_some_and(|expected_per_id| {
                                    expected_per_id != remote_peer_id
                                }) {
                                    *state = P2pNetworkNoiseStateInner::Error(
                                        NoiseError::RemotePeerIdMismatch,
                                    );
                                } else {
                                    *state = P2pNetworkNoiseStateInner::Done {
                                        incoming: false,
                                        send_key,
                                        recv_key,
                                        recv_nonce: 0,
                                        send_nonce: 0,
                                        remote_pk,
                                        remote_peer_id,
                                    };
                                }
                            }
                            (Err(error), Some(_)) => {
                                *state = P2pNetworkNoiseStateInner::Error(error);
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    P2pNetworkNoiseStateInner::Responder(r) => {
                        if let Some(chunk) = r.generate(data) {
                            noise_state.outgoing_chunks.push_back(vec![chunk.into()]);
                        }
                    }
                    // TODO: report error
                    _ => {}
                }

                let mut outgoing = noise_state.outgoing_chunks.clone();
                let error = noise_state.as_error();
                let dispatcher = state_context.into_dispatcher();

                if let Some(error) = error {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr: *addr,
                        error: P2pNetworkConnectionError::Noise(error),
                    });
                } else {
                    while let Some(data) = outgoing.pop_front() {
                        dispatcher.push(P2pNetworkNoiseAction::OutgoingChunk { addr: *addr, data });
                    }
                }
                Ok(())
            }
            P2pNetworkNoiseAction::OutgoingDataSelectMux { data, addr } => {
                let Some(P2pNetworkNoiseStateInner::Done {
                    send_key,
                    send_nonce,
                    ..
                }) = &mut noise_state.inner
                else {
                    bug_condition!("action {:?}: no inner state", action);
                    return Ok(());
                };

                let aead = ChaCha20Poly1305::new(&send_key.0.into());

                let mut chunk = Vec::with_capacity(18 + data.len());
                chunk.extend_from_slice(&((data.len() + 16) as u16).to_be_bytes());
                chunk.extend_from_slice(data);

                let mut nonce = GenericArray::default();
                nonce[4..].clone_from_slice(&send_nonce.to_le_bytes());
                *send_nonce += 1;

                match aead.encrypt_in_place_detached(&nonce, &[], &mut chunk[2..(2 + data.len())]) {
                    Ok(tag) => {
                        chunk.extend_from_slice(&tag);
                        noise_state.outgoing_chunks.push_back(vec![chunk.into()]);
                    }
                    Err(_) => {
                        noise_state.inner =
                            Some(P2pNetworkNoiseStateInner::Error(NoiseError::Encryption));
                    }
                };

                let outgoing = noise_state.outgoing_chunks.front().cloned();
                let error = noise_state.as_error();
                let dispatcher = state_context.into_dispatcher();

                if let Some(error) = error {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr: *addr,
                        error: P2pNetworkConnectionError::Noise(error),
                    });
                } else if let Some(data) = outgoing {
                    dispatcher
                        .push(P2pNetworkNoiseAction::OutgoingChunkSelectMux { addr: *addr, data })
                }

                Ok(())
            }
            P2pNetworkNoiseAction::DecryptedData {
                addr,
                peer_id,
                data,
            } => {
                noise_state.decrypted_chunks.pop_front();

                let remote_peer_id = noise_state.remote_peer_id();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSelectAction::IncomingDataMux {
                    addr: *addr,
                    peer_id: (*peer_id).or(remote_peer_id),
                    data: data.clone(),
                    fin: false,
                });

                Ok(())
            }
            P2pNetworkNoiseAction::HandshakeDone {
                addr,
                peer_id,
                incoming,
            } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSelectAction::Init {
                    addr: *addr,
                    kind: SelectKind::Multiplexing(*peer_id),
                    incoming: *incoming,
                });
                Ok(())
            }
        }
    }
}

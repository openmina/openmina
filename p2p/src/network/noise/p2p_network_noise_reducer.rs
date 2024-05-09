use chacha20poly1305::{aead::generic_array::GenericArray, AeadInPlace, ChaCha20Poly1305, KeyInit};

use self::p2p_network_noise_state::ResponderConsumeOutput;

use super::*;

use super::p2p_network_noise_state::{
    NoiseError, NoiseState, P2pNetworkNoiseState, P2pNetworkNoiseStateInitiator,
    P2pNetworkNoiseStateInner, P2pNetworkNoiseStateResponder, ResponderOutput,
};

impl P2pNetworkNoiseState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkNoiseAction>) {
        match action.action() {
            P2pNetworkNoiseAction::Init {
                incoming,
                ephemeral_sk,
                static_sk,
                signature,
                ..
            } => {
                let esk = ephemeral_sk.clone();
                let epk = esk.pk();
                let ssk = static_sk.clone();
                let spk = ssk.pk();
                let payload = signature.clone();

                self.inner = if *incoming {
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
                    self.outgoing_chunks.push_back(vec![chunk.into()]);

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
                }
            }
            P2pNetworkNoiseAction::IncomingData { data, .. } => {
                self.buffer.extend_from_slice(data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if buf.len() >= 2 {
                        let len = u16::from_be_bytes(buf[..2].try_into().expect("cannot fail"));
                        let full_len = 2 + len as usize;
                        if buf.len() >= full_len {
                            self.incoming_chunks.push_back(buf[..full_len].to_vec());
                            offset += full_len;

                            continue;
                        }
                    }
                    break;
                }
                self.buffer = self.buffer[offset..].to_vec();
            }
            P2pNetworkNoiseAction::IncomingChunk { .. } => {
                let Some(state) = &mut self.inner else {
                    return;
                };
                if let Some(mut chunk) = self.incoming_chunks.pop_front() {
                    match state {
                        P2pNetworkNoiseStateInner::Initiator(i) => match i.consume(&mut chunk) {
                            Ok(_) => {
                                // self.handshake_optimized = remote_payload.is_some();
                                // if let Some(remote_payload) = remote_payload {
                                //     self.decrypted_chunks
                                //         .push_back(remote_payload.to_vec().into());
                                // }
                            }
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(dbg!(err)),
                        },
                        P2pNetworkNoiseStateInner::Responder(o) => match o.consume(&mut chunk) {
                            Ok(None) => {}
                            Ok(Some(ResponderConsumeOutput {
                                output: ResponderOutput { remote_pk, .. },
                                ..
                            })) if remote_pk == self.local_pk => {
                                *state = P2pNetworkNoiseStateInner::Error(dbg!(
                                    NoiseError::SelfConnection
                                ));
                            }
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
                                *state = P2pNetworkNoiseStateInner::Done {
                                    incoming: true,
                                    send_key,
                                    recv_key,
                                    recv_nonce: 0,
                                    send_nonce: 0,
                                    remote_pk,
                                    remote_peer_id,
                                };
                                // self.handshake_optimized = remote_payload.is_some();
                                // if let Some(remote_payload) = remote_payload {
                                //     self.decrypted_chunks
                                //         .push_back(remote_payload.to_vec().into());
                                // }
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
                                    self.decrypted_chunks.push_back(data.to_vec().into());
                                }
                            }
                        }
                        P2pNetworkNoiseStateInner::Error(_) => {}
                    }
                }
            }
            P2pNetworkNoiseAction::OutgoingChunk { .. } => {
                self.outgoing_chunks.pop_front();
            }
            P2pNetworkNoiseAction::OutgoingData { data, .. } => {
                let Some(state) = &mut self.inner else {
                    return;
                };
                if data.is_empty() && self.handshake_optimized {
                    return;
                }
                match state {
                    P2pNetworkNoiseStateInner::Done {
                        send_key,
                        send_nonce,
                        ..
                    } => {
                        let aead = ChaCha20Poly1305::new(&send_key.0.into());
                        let chunk_max_size = u16::MAX as usize - 19;
                        let chunks = data
                            .chunks(chunk_max_size)
                            .map(|data| {
                                let mut chunk = Vec::with_capacity(18 + data.len());
                                chunk.extend_from_slice(&((data.len() + 16) as u16).to_be_bytes());
                                chunk.extend_from_slice(data);

                                let mut nonce = GenericArray::default();
                                nonce[4..].clone_from_slice(&send_nonce.to_le_bytes());
                                *send_nonce += 1;

                                let tag = aead
                                    .encrypt_in_place_detached(
                                        &nonce,
                                        &[],
                                        &mut chunk[2..(2 + data.len())],
                                    )
                                    .expect("cannot fail");
                                chunk.extend_from_slice(&tag);
                                chunk.into()
                            })
                            .collect();
                        self.outgoing_chunks.push_back(chunks);
                    }
                    P2pNetworkNoiseStateInner::Initiator(i) => {
                        if let (Some((chunk, (send_key, recv_key))), Some(remote_pk)) =
                            (i.generate(data), i.remote_pk.clone())
                        {
                            self.outgoing_chunks.push_back(vec![chunk.into()]);
                            let remote_peer_id = remote_pk.peer_id();
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
                    P2pNetworkNoiseStateInner::Responder(r) => {
                        if let Some(chunk) = r.generate(data) {
                            self.outgoing_chunks.push_back(vec![chunk.into()]);
                        }
                    }
                    // TODO: report error
                    _ => {}
                }
            }
            P2pNetworkNoiseAction::DecryptedData { .. } => {
                self.decrypted_chunks.pop_front();
            }
            P2pNetworkNoiseAction::HandshakeDone { .. } => {}
        }
    }
}

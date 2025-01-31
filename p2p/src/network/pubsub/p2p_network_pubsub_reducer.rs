use std::{collections::btree_map::Entry, time::Duration};

use binprot::BinProtRead;
use mina_p2p_messages::{
    gossip::{self, GossipNetMessageV2},
    v2::NetworkPoolSnarkPoolDiffVersionedStableV2,
};
use openmina_core::{block::BlockWithHash, bug_condition, fuzz_maybe, fuzzed_maybe, Substate};
use redux::{Dispatcher, Timestamp};

use crate::{
    channels::{snark::P2pChannelsSnarkAction, transaction::P2pChannelsTransactionAction},
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    peer::P2pPeerAction,
    Data, P2pConfig, P2pNetworkYamuxAction, P2pState, PeerId,
};

use super::{
    p2p_network_pubsub_state::{
        source_from_message, P2pNetworkPubsubClientMeshAddingState,
        P2pNetworkPubsubMessageCacheMessage,
    },
    pb::{self, Message},
    P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubEffectfulAction,
    P2pNetworkPubsubMessageCacheId, P2pNetworkPubsubState, TOPIC,
};

const MAX_MESSAGE_KEEP_DURATION: Duration = Duration::from_secs(300);

impl P2pNetworkPubsubState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, Self>,
        action: redux::ActionWithMeta<P2pNetworkPubsubAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let pubsub_state = state_context.get_substate_mut()?;
        let (action, meta) = action.split();
        let time = meta.time();

        match action {
            P2pNetworkPubsubAction::NewStream {
                incoming: true,
                peer_id,
                addr,
                protocol,
                ..
            } => {
                let entry = pubsub_state.clients.entry(peer_id);
                // preserve it
                let outgoing_stream_id = match &entry {
                    Entry::Occupied(v) => v.get().outgoing_stream_id,
                    Entry::Vacant(_) => None,
                };
                let state = entry.or_insert_with(|| P2pNetworkPubsubClientState {
                    protocol,
                    addr,
                    outgoing_stream_id,
                    message: pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: None,
                    },
                    cache: Default::default(),
                    buffer: vec![],
                    incoming_messages: vec![],
                });
                state.protocol = protocol;
                state.addr = addr;

                pubsub_state
                    .topics
                    .entry(super::TOPIC.to_owned())
                    .or_default()
                    .insert(peer_id, Default::default());

                Ok(())
            }
            P2pNetworkPubsubAction::NewStream {
                incoming: false,
                peer_id,
                stream_id,
                addr,
                protocol,
            } => {
                let state = pubsub_state.clients.entry(peer_id).or_insert_with(|| {
                    P2pNetworkPubsubClientState {
                        protocol,
                        addr,
                        outgoing_stream_id: Some(stream_id),
                        message: pb::Rpc {
                            subscriptions: vec![],
                            publish: vec![],
                            control: None,
                        },
                        cache: Default::default(),
                        buffer: vec![],
                        incoming_messages: vec![],
                    }
                });
                state.outgoing_stream_id = Some(stream_id);
                state.protocol = protocol;
                state.addr = addr;

                pubsub_state
                    .topics
                    .entry(TOPIC.to_owned())
                    .or_default()
                    .insert(peer_id, Default::default());

                if let Some(state) = pubsub_state.clients.get_mut(&peer_id) {
                    state.message.subscriptions.push(pb::rpc::SubOpts {
                        subscribe: Some(true),
                        topic_id: Some(TOPIC.to_owned()),
                    });
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let config: &P2pConfig = state.substate()?;
                let state: &P2pNetworkPubsubState = state.substate()?;

                let Some(map) = state.topics.get(TOPIC) else {
                    // must have this topic already
                    return Ok(());
                };
                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage { peer_id });
                let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                if mesh_size < config.meshsub.outbound_degree_desired {
                    dispatcher.push(P2pNetworkPubsubAction::Graft {
                        peer_id,
                        topic_id: TOPIC.to_owned(),
                    });
                }

                Ok(())
            }
            P2pNetworkPubsubAction::IncomingData {
                peer_id,
                data,
                seen_limit,
                addr,
                ..
            } => {
                pubsub_state.reduce_incoming_data(&peer_id, data, meta.time())?;

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pNetworkPubsubAction::ValidateIncomingMessages {
                    peer_id,
                    seen_limit,
                    addr,
                });

                Ok(())
            }
            P2pNetworkPubsubAction::ValidateIncomingMessages {
                peer_id,
                seen_limit,
                addr,
            } => {
                let Some(state) = pubsub_state.clients.get_mut(&peer_id) else {
                    // TODO: investigate, cannot reproduce this
                    // bug_condition!("{:?} not found in state.clients", peer_id);
                    return Ok(());
                };
                let messages = std::mem::take(&mut state.incoming_messages);

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pNetworkPubsubEffectfulAction::ValidateIncomingMessages {
                    peer_id,
                    seen_limit,
                    addr,
                    messages,
                });

                Ok(())
            }
            P2pNetworkPubsubAction::IncomingMessage {
                peer_id,
                message,
                seen_limit,
            } => {
                // Check that if we can extract source from message, this is pre check
                if source_from_message(&message).is_err() {
                    let dispatcher = state_context.into_dispatcher();
                    dispatcher.push(P2pNetworkPubsubAction::RejectMessage {
                        message_id: None,
                        peer_id: Some(peer_id),
                        reason: "Invalid originator in message".to_owned(),
                    });
                    return Ok(());
                }

                // Check result later to ensure we always dispatch the cleanup action
                let reduce_incoming_result =
                    pubsub_state.reduce_incoming_message(&message, seen_limit);

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = global_state.substate()?;
                let state: &Self = global_state.substate()?;

                dispatcher.push(P2pNetworkPubsubAction::IncomingMessageCleanup { peer_id });

                let message_content = reduce_incoming_result?;

                for (topic_id, map) in &state.topics {
                    let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                    let could_accept = mesh_size < p2p_state.config.meshsub.outbound_degree_high;

                    if !could_accept {
                        if let Some(topic_state) = map.get(&peer_id) {
                            if topic_state.on_mesh() {
                                let topic_id = topic_id.clone();
                                dispatcher.push(P2pNetworkPubsubAction::Prune { peer_id, topic_id })
                            }
                        }
                    }
                }

                // This happens if message was already seen
                if let Some(message_content) = message_content {
                    dispatcher.push(P2pNetworkPubsubAction::HandleIncomingMessage {
                        message,
                        message_content,
                        peer_id,
                    });
                } else {
                    dispatcher.push(P2pNetworkPubsubAction::IgnoreMessage {
                        message_id: None,
                        reason: "Message already seen".to_owned(),
                    });
                };

                Ok(())
            }
            P2pNetworkPubsubAction::HandleIncomingMessage {
                message,
                message_content,
                peer_id,
            } => {
                let Ok(message_id) =
                    pubsub_state
                        .mcache
                        .put(message, message_content, peer_id, time)
                else {
                    bug_condition!("Unable to add message to `mcache`");
                    return Ok(());
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = p2p_state.callbacks.on_p2p_pubsub_message_received.clone() {
                    dispatcher.push_callback(callback, message_id);
                } else {
                    dispatcher.push(P2pNetworkPubsubAction::ValidateIncomingMessage { message_id });
                }
                Ok(())
            }
            P2pNetworkPubsubAction::IncomingMessageCleanup { peer_id } => {
                pubsub_state.clear_incoming();

                let Some(client_state) = pubsub_state.clients.get_mut(&peer_id) else {
                    bug_condition!(
                        "State not found for action P2pNetworkPubsubAction::IncomingMessageCleanup"
                    );
                    return Ok(());
                };

                client_state.clear_incoming();

                Ok(())
            }
            // we want to add peer to our mesh
            P2pNetworkPubsubAction::Graft { peer_id, topic_id } => {
                let Some(state) = pubsub_state
                    .topics
                    .get_mut(&topic_id)
                    .and_then(|m| m.get_mut(&peer_id))
                else {
                    return Ok(());
                };
                state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;

                if let Some(state) = pubsub_state.clients.get_mut(&peer_id) {
                    let control = state
                        .message
                        .control
                        .get_or_insert_with(|| pb::ControlMessage {
                            ihave: vec![],
                            iwant: vec![],
                            graft: vec![],
                            prune: vec![],
                        });
                    control.graft.push(pb::ControlGraft {
                        topic_id: Some(topic_id),
                    });
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage { peer_id });
                Ok(())
            }
            P2pNetworkPubsubAction::Prune { peer_id, topic_id } => {
                let Some(state) = pubsub_state
                    .topics
                    .get_mut(&topic_id)
                    .and_then(|m| m.get_mut(&peer_id))
                else {
                    bug_condition!("State not found for action: `P2pNetworkPubsubAction::Prune`");
                    return Ok(());
                };
                state.mesh = P2pNetworkPubsubClientMeshAddingState::WeRefused;

                if let Some(state) = pubsub_state.clients.get_mut(&peer_id) {
                    let control = state
                        .message
                        .control
                        .get_or_insert_with(|| pb::ControlMessage {
                            ihave: vec![],
                            iwant: vec![],
                            graft: vec![],
                            prune: vec![],
                        });
                    control.prune.push(pb::ControlPrune {
                        topic_id: Some(topic_id),
                        peers: vec![pb::PeerInfo {
                            peer_id: None,
                            signed_peer_record: None,
                        }],
                        backoff: None,
                    });
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage { peer_id });
                Ok(())
            }
            P2pNetworkPubsubAction::OutgoingMessage { peer_id } => {
                let msg = if let Some(v) = pubsub_state.clients.get_mut(&peer_id) {
                    &v.message
                } else {
                    bug_condition!(
                        "Invalid state for action: `P2pNetworkPubsubAction::OutgoingMessage`"
                    );
                    return Ok(());
                };

                let mut data = vec![];
                let result = prost::Message::encode_length_delimited(msg, &mut data)
                    .map(|_| data)
                    .map_err(|_| msg.clone());

                let dispatcher = state_context.into_dispatcher();

                match result {
                    Err(msg) => {
                        dispatcher
                            .push(P2pNetworkPubsubAction::OutgoingMessageError { msg, peer_id });
                    }
                    Ok(data) => {
                        dispatcher.push(P2pNetworkPubsubAction::OutgoingData {
                            data: Data::from(data),
                            peer_id,
                        });
                    }
                }

                // Important to avoid leaking state
                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessageClear { peer_id });

                Ok(())
            }
            P2pNetworkPubsubAction::OutgoingMessageClear { peer_id } => {
                if let Some(v) = pubsub_state.clients.get_mut(&peer_id) {
                    v.message = pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: None,
                    };
                } else {
                    bug_condition!(
                        "Invalid state for action: `P2pNetworkPubsubAction::OutgoingMessageClear`"
                    );
                };
                Ok(())
            }
            P2pNetworkPubsubAction::OutgoingMessageError { .. } => Ok(()),
            P2pNetworkPubsubAction::WebRtcRebroadcast { message } => {
                let data = match super::encode_message(&message) {
                    Err(err) => {
                        bug_condition!("binprot serialization error: {err}");
                        return Ok(());
                    }
                    Ok(data) => data,
                };

                let mut source_sk = super::webrtc_source_sk_from_bytes(&data[8..]);
                let source_peer_id = source_sk.public_key().peer_id();
                let message_id = P2pNetworkPubsubMessageCacheId {
                    source: libp2p_identity::PeerId::try_from(source_peer_id).unwrap(),
                    seqno: 0,
                };
                let mut msg = pb::Message {
                    from: Some(message_id.source.to_bytes().to_vec()),
                    data: Some(data),
                    seqno: Some(message_id.seqno.to_be_bytes().to_vec()),
                    topic: super::TOPIC.to_owned(),
                    signature: None,
                    key: None,
                };

                msg.signature = match source_sk.libp2p_pubsub_pb_message_sign(&msg) {
                    Err(err) => {
                        bug_condition!("pubsub prost encode error: {err}");
                        return Ok(());
                    }
                    Ok(sig) => Some(sig.to_bytes().to_vec()),
                };

                let message_state = match &message {
                    GossipNetMessageV2::NewState(block) => {
                        P2pNetworkPubsubMessageCacheMessage::PreValidatedBlockMessage {
                            block_hash: block.try_hash()?,
                            message: msg,
                            peer_id: source_peer_id,
                            time,
                        }
                    }
                    _ => P2pNetworkPubsubMessageCacheMessage::PreValidated {
                        message: msg,
                        peer_id: source_peer_id,
                        time,
                    },
                };

                pubsub_state.mcache.map.insert(message_id, message_state);

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pNetworkPubsubAction::BroadcastValidatedMessage {
                    message_id: super::BroadcastMessageId::MessageId { message_id },
                });

                Ok(())
            }
            P2pNetworkPubsubAction::Broadcast { message } => {
                let data = match super::encode_message(&message) {
                    Err(err) => {
                        bug_condition!("binprot serialization error: {err}");
                        return Ok(());
                    }
                    Ok(data) => data,
                };

                Self::prepare_to_sign(state_context, data)
            }
            P2pNetworkPubsubAction::Sign {
                seqno,
                author,
                data,
                topic,
            } => {
                pubsub_state.seq += 1;

                let libp2p_peer_id =
                    libp2p_identity::PeerId::try_from(author).expect("valid peer_id"); // This can't happen unless something is broken in the configuration
                pubsub_state.to_sign.push_back(pb::Message {
                    from: Some(libp2p_peer_id.to_bytes()),
                    data: Some(data.0.into_vec()),
                    seqno: Some(seqno.to_be_bytes().to_vec()),
                    topic: topic.clone(),
                    signature: None,
                    key: None,
                });

                let to_sign = pubsub_state.to_sign.front().cloned();
                let Some(message) = to_sign else {
                    bug_condition!("Message not found");
                    return Ok(());
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPubsubEffectfulAction::Sign { author, message });
                Ok(())
            }
            P2pNetworkPubsubAction::SignError { .. } => {
                let _ = pubsub_state.to_sign.pop_front();
                Ok(())
            }
            P2pNetworkPubsubAction::BroadcastSigned { signature } => {
                if let Some(mut message) = pubsub_state.to_sign.pop_front() {
                    message.signature = Some(signature.0.to_vec());
                    pubsub_state
                        .clients
                        .iter_mut()
                        .for_each(|(_, state)| state.publish(&message));
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::broadcast(dispatcher, state)
            }
            P2pNetworkPubsubAction::OutgoingData { mut data, peer_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &Self = state.substate()?;

                let Some(state) = state.clients.get(&peer_id) else {
                    bug_condition!(
                        "Missing state for action: `P2pNetworkPubsubAction::OutgoingData`"
                    );
                    return Ok(());
                };
                fuzz_maybe!(&mut data, crate::fuzzer::mutate_pubsub);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                if let Some(stream_id) = state.outgoing_stream_id.as_ref().copied() {
                    dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                        addr: state.addr,
                        stream_id,
                        data,
                        flags,
                    });
                }
                Ok(())
            }
            P2pNetworkPubsubAction::ValidateIncomingMessage { message_id } => {
                let Some(message) = pubsub_state.mcache.map.remove(&message_id) else {
                    bug_condition!("Message with id: {:?} not found", message_id);
                    return Ok(());
                };

                let P2pNetworkPubsubMessageCacheMessage::Init {
                    message,
                    content,
                    time,
                    peer_id,
                } = message
                else {
                    bug_condition!(
                        "`P2pNetworkPubsubAction::ValidateIncomingMessage` called on invalid state"
                    );
                    return Ok(());
                };

                let new_message_state = match &content {
                    GossipNetMessageV2::NewState(block) => {
                        let block_hash = block.try_hash()?;
                        P2pNetworkPubsubMessageCacheMessage::PreValidatedBlockMessage {
                            block_hash,
                            message,
                            peer_id,
                            time,
                        }
                    }
                    _ => P2pNetworkPubsubMessageCacheMessage::PreValidated {
                        message,
                        peer_id,
                        time,
                    },
                };
                pubsub_state
                    .mcache
                    .map
                    .insert(message_id, new_message_state);

                let dispatcher = state_context.into_dispatcher();

                match content {
                    GossipNetMessageV2::NewState(block) => {
                        let best_tip = BlockWithHash::try_new(block.clone())?;
                        dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                        return Ok(());
                    }
                    GossipNetMessageV2::TransactionPoolDiff { message, nonce } => {
                        let nonce = nonce.as_u32();
                        for transaction in message.0 {
                            dispatcher.push(P2pChannelsTransactionAction::Libp2pReceived {
                                peer_id,
                                transaction: Box::new(transaction),
                                nonce,
                            });
                        }
                    }
                    GossipNetMessageV2::SnarkPoolDiff {
                        message: NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(work),
                        nonce,
                    } => {
                        dispatcher.push(P2pChannelsSnarkAction::Libp2pReceived {
                            peer_id,
                            snark: Box::new(work.1.into()),
                            nonce: nonce.as_u32(),
                        });
                    }
                    _ => {}
                }

                dispatcher.push(P2pNetworkPubsubAction::BroadcastValidatedMessage {
                    message_id: super::BroadcastMessageId::MessageId { message_id },
                });
                Ok(())
            }
            P2pNetworkPubsubAction::BroadcastValidatedMessage { message_id } => {
                let Some((mcache_message_id, message)) =
                    pubsub_state.mcache.get_message_id_and_message(&message_id)
                else {
                    bug_condition!("Message with id: {:?} not found", message_id);
                    return Ok(());
                };
                let raw_message = message.message().clone();
                let peer_id = message.peer_id();

                pubsub_state.reduce_incoming_validated_message(
                    mcache_message_id,
                    peer_id,
                    &raw_message,
                );

                let Some((_message_id, message)) =
                    pubsub_state.mcache.get_message_id_and_message(&message_id)
                else {
                    bug_condition!("Message with id: {:?} not found", message_id);
                    return Ok(());
                };

                *message = P2pNetworkPubsubMessageCacheMessage::Validated {
                    message: raw_message,
                    peer_id,
                    time: message.time(),
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                Self::broadcast(dispatcher, state)
            }
            P2pNetworkPubsubAction::PruneMessages {} => {
                let messages = pubsub_state
                    .mcache
                    .map
                    .iter()
                    .filter_map(|(message_id, message)| {
                        if message.time() + MAX_MESSAGE_KEEP_DURATION > time {
                            Some(message_id.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for message_id in messages {
                    pubsub_state.mcache.remove_message(message_id);
                }
                Ok(())
            }
            P2pNetworkPubsubAction::RejectMessage {
                message_id,
                peer_id,
                ..
            } => {
                let mut peer_id = peer_id;
                if let Some(message_id) = message_id {
                    let Some((_message_id, message)) =
                        pubsub_state.mcache.get_message_id_and_message(&message_id)
                    else {
                        bug_condition!("Message not found for id: {:?}", message_id);
                        return Ok(());
                    };

                    if peer_id.is_none() {
                        peer_id = Some(message.peer_id());
                    }

                    pubsub_state.mcache.remove_message(_message_id);
                }

                let dispatcher = state_context.into_dispatcher();

                if let Some(peer_id) = peer_id {
                    dispatcher.push(P2pDisconnectionAction::Init {
                        peer_id,
                        reason: P2pDisconnectionReason::InvalidMessage,
                    });
                }

                Ok(())
            }
            P2pNetworkPubsubAction::IgnoreMessage { .. } => Ok(()),
        }
    }

    fn prepare_to_sign<Action, State>(
        mut state_context: Substate<Action, State, Self>,
        buffer: Vec<u8>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let pubsub_state = state_context.get_substate_mut()?;

        let mut seqno = pubsub_state.seq;
        let (dispatcher, state) = state_context.into_dispatcher_and_state();
        let config: &P2pConfig = state.substate()?;
        seqno += config.meshsub.initial_time.as_nanos() as u64;

        dispatcher.push(P2pNetworkPubsubAction::Sign {
            seqno,
            author: config.identity_pub_key.peer_id(),
            data: buffer.into(),
            topic: super::TOPIC.to_owned(),
        });

        Ok(())
    }

    fn reduce_incoming_validated_message(
        &mut self,
        message_id: P2pNetworkPubsubMessageCacheId,
        peer_id: PeerId,
        message: &Message,
    ) {
        let topic = self.topics.entry(message.topic.clone()).or_default();

        self.clients
            .iter_mut()
            .filter(|(c, _)| {
                // don't send back to who sent this
                *c != &peer_id
            })
            .for_each(|(c, state)| {
                let Some(topic_state) = topic.get(c) else {
                    return;
                };
                if topic_state.on_mesh() {
                    state.publish(message)
                } else {
                    let ctr = state.message.control.get_or_insert_with(Default::default);
                    ctr.ihave.push(pb::ControlIHave {
                        topic_id: Some(message.topic.clone()),
                        message_ids: vec![message_id.to_raw_bytes()],
                    })
                }
            });
    }

    /// Processes an incoming message by checking for duplicates and deserializing its contents.
    ///
    /// This function performs two main operations:
    /// 1. Deduplication: Tracks recently seen messages using their signatures to avoid processing duplicates
    /// 2. Deserialization: Converts valid message data into a `GossipNetMessageV2` structure
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to process
    /// * `seen_limit` - Maximum number of message signatures to keep in the deduplication cache
    ///
    /// # Returns
    ///
    /// * `Ok(Some(GossipNetMessageV2))` - Successfully processed and deserialized message
    /// * `Ok(None)` - Message was a duplicate (already seen)
    /// * `Err(String)` - Error during processing (invalid message format or deserialization failure)
    ///
    #[inline(never)]
    fn reduce_incoming_message(
        &mut self,
        message: &Message,
        seen_limit: usize,
    ) -> Result<Option<GossipNetMessageV2>, String> {
        let Some(signature) = &message.signature else {
            bug_condition!("Validation failed: missing signature");
            return Ok(None);
        };

        // skip recently seen message
        if !self.seen.contains(signature) {
            self.seen.push_back(signature.clone());
            // keep only last `n` to avoid memory leak
            if self.seen.len() > seen_limit {
                self.seen.pop_front();
            }
        } else {
            return Ok(None);
        }

        match &message.data {
            Some(data) if data.len() > 8 => {
                let mut slice = &data[8..];
                Ok(Some(
                    gossip::GossipNetMessageV2::binprot_read(&mut slice)
                        .map_err(|e| format!("Invalid `GossipNetMessageV2` message, error: {e}"))?,
                ))
            }
            _ => Err("Invalid message".to_owned()),
        }
    }

    fn combined_with_pending_buffer<'a>(buffer: &'a mut Vec<u8>, data: &'a [u8]) -> &'a [u8] {
        if buffer.is_empty() {
            // Nothing pending, we can use the data directly
            data
        } else {
            buffer.extend_from_slice(data);
            buffer.as_slice()
        }
    }

    /// Processes incoming data from a peer, handling subscriptions, control messages,
    /// and message broadcasting within the P2P pubsub system.
    fn reduce_incoming_data(
        &mut self,
        peer_id: &PeerId,
        data: Data,
        timestamp: Timestamp,
    ) -> Result<(), String> {
        let Some(client_state) = self.clients.get_mut(peer_id) else {
            // TODO: investigate, cannot reproduce this
            // bug_condition!("State not found for action: P2pNetworkPubsubAction::IncomingData");
            return Ok(());
        };

        // Data may be part of a partial message we received before.
        let slice = Self::combined_with_pending_buffer(&mut client_state.buffer, &data);

        match <pb::Rpc as prost::Message>::decode_length_delimited(slice) {
            Ok(decoded) => {
                client_state.clear_buffer();
                client_state.incoming_messages.extend(decoded.publish);

                let subscriptions = decoded.subscriptions;
                let control = decoded.control.unwrap_or_default();

                self.update_subscriptions(peer_id, subscriptions);
                self.apply_control_commands(peer_id, &control);
                self.respond_to_iwant_requests(peer_id, &control.iwant);
                self.process_ihave_messages(peer_id, control.ihave, timestamp);
            }
            Err(err) => {
                // NOTE: not the ideal way to check for errors, but `prost` doesn't provide
                // a better alternative, so we must check the message contents.
                if err.to_string().contains("buffer underflow") && client_state.buffer.is_empty() {
                    // Incomplete data, keep in buffer, should be completed later
                    client_state.buffer = data.to_vec();
                } else {
                    // Clear the buffer for other decoding errors, otherwise this will cause issues
                    // with any data we receive later.
                    client_state.clear_buffer();
                }
            }
        }

        Ok(())
    }

    fn update_subscriptions(&mut self, peer_id: &PeerId, subscriptions: Vec<pb::rpc::SubOpts>) {
        // Update subscription status based on incoming subscription requests.
        for subscription in &subscriptions {
            let topic_id = subscription.topic_id().to_owned();
            let topic = self.topics.entry(topic_id).or_default();

            if subscription.subscribe() {
                topic.entry(*peer_id).or_default();
            } else {
                topic.remove(peer_id);
            }
        }
    }

    /// Applies control commands (`graft` and `prune`) to manage the peer's mesh states within topics.
    fn apply_control_commands(&mut self, peer_id: &PeerId, control: &pb::ControlMessage) {
        // Apply graft commands to add the peer to specific topic meshes.
        for graft in &control.graft {
            if let Some(mesh_state) = self
                .topics
                .get_mut(graft.topic_id())
                .and_then(|m| m.get_mut(peer_id))
            {
                mesh_state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;
            }
        }

        // Apply prune commands to remove the peer from specific topic meshes.
        for prune in &control.prune {
            if let Some(mesh_state) = self
                .topics
                .get_mut(prune.topic_id())
                .and_then(|m| m.get_mut(peer_id))
            {
                mesh_state.mesh = P2pNetworkPubsubClientMeshAddingState::TheyRefused;
            }
        }
    }

    fn respond_to_iwant_requests(&mut self, peer_id: &PeerId, iwant_requests: &[pb::ControlIWant]) {
        // Respond to iwant requests by publishing available messages from the cache.
        for iwant in iwant_requests {
            for msg_id in &iwant.message_ids {
                if let Some(msg) = self.mcache.get_message_from_raw_message_id(msg_id) {
                    if let Some(client) = self.clients.get_mut(peer_id) {
                        client.publish(msg.message());
                    }
                }
            }
        }
    }

    fn process_ihave_messages(
        &mut self,
        peer_id: &PeerId,
        ihave_messages: Vec<pb::ControlIHave>,
        timestamp: Timestamp,
    ) {
        // Process ihave messages by determining which available messages the client wants.
        for ihave in ihave_messages {
            if self.clients.contains_key(peer_id) {
                let message_ids = ihave
                    .message_ids
                    .into_iter()
                    .filter(|message_id| self.filter_iwant_message_ids(message_id, timestamp))
                    .collect::<Vec<_>>();

                let Some(client) = self.clients.get_mut(peer_id) else {
                    bug_condition!("process_ihave_messages: State not found for {}", peer_id);
                    return;
                };

                // Queue the desired message IDs for the client to request.
                let ctr = client.message.control.get_or_insert_with(Default::default);
                ctr.iwant.push(pb::ControlIWant { message_ids })
            }
        }
    }

    fn broadcast<Action, State>(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let state: &P2pNetworkPubsubState = state.substate()?;

        for peer_id in state
            .clients
            .iter()
            .filter(|(_, s)| !s.message_is_empty())
            .map(|(peer_id, _)| *peer_id)
        {
            dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage { peer_id });
        }

        Ok(())
    }
}

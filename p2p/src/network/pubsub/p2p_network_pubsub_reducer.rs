use std::collections::btree_map::Entry;

use binprot::BinProtRead;
use mina_p2p_messages::{gossip, v2};
use openmina_core::{block::BlockWithHash, bug_condition, fuzz_maybe, fuzzed_maybe, Substate};
use redux::{Dispatcher, Timestamp};

use crate::{
    channels::{snark::P2pChannelsSnarkAction, transaction::P2pChannelsTransactionAction},
    peer::P2pPeerAction,
    Data, P2pConfig, P2pNetworkYamuxAction, P2pState, PeerId,
};

use super::{
    p2p_network_pubsub_state::P2pNetworkPubsubClientMeshAddingState,
    pb::{self, Message},
    P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubEffectfulAction,
    P2pNetworkPubsubState, TOPIC,
};

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

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                let state = &p2p_state.network.scheduler.broadcast_state;
                let Some(state) = state.clients.get(&peer_id) else {
                    // TODO: investigate, cannot reproduce this
                    // bug_condition!("{:?} not found in state.clients", peer_id);
                    return Ok(());
                };

                // TODO: try to reuse data instead of cloning all message
                let messages = state.incoming_messages.clone();
                dispatcher.push(P2pNetworkPubsubEffectfulAction::IncomingData {
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
                // Check result later to ensure we always dispatch the cleanup action
                let reduce_incoming_result =
                    pubsub_state.reduce_incoming_message(peer_id, message, seen_limit);

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();

                dispatcher.push(P2pNetworkPubsubAction::IncomingMessageCleanup { peer_id });

                reduce_incoming_result?;

                let state: &Self = global_state.substate()?;
                let config: &P2pConfig = global_state.substate()?;

                for (topic_id, map) in &state.topics {
                    let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                    let could_accept = mesh_size < config.meshsub.outbound_degree_high;

                    if !could_accept {
                        if let Some(topic_state) = map.get(&peer_id) {
                            if topic_state.on_mesh() {
                                let topic_id = topic_id.clone();
                                dispatcher.push(P2pNetworkPubsubAction::Prune { peer_id, topic_id })
                            }
                        }
                    }
                }

                if let Err(error) = Self::broadcast(dispatcher, global_state) {
                    bug_condition!(
                        "Failure when trying to broadcast incoming pubsub message: {error}"
                    );
                };

                if let Some((_, block)) = state.incoming_block.as_ref() {
                    let best_tip = BlockWithHash::try_new(block.clone())?;
                    dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                }
                for (transaction, nonce) in &state.incoming_transactions {
                    dispatcher.push(P2pChannelsTransactionAction::Libp2pReceived {
                        peer_id,
                        transaction: Box::new(transaction.clone()),
                        nonce: *nonce,
                    });
                }
                for (snark, nonce) in &state.incoming_snarks {
                    dispatcher.push(P2pChannelsSnarkAction::Libp2pReceived {
                        peer_id,
                        snark: Box::new(snark.clone()),
                        nonce: *nonce,
                    });
                }
                Ok(())
            }
            P2pNetworkPubsubAction::IncomingMessageCleanup { peer_id } => {
                pubsub_state.incoming_transactions.clear();
                pubsub_state.incoming_snarks.clear();

                pubsub_state.incoming_transactions.shrink_to(0x20);
                pubsub_state.incoming_snarks.shrink_to(0x20);

                pubsub_state.incoming_block = None;

                let Some(client_state) = pubsub_state.clients.get_mut(&peer_id) else {
                    bug_condition!(
                        "State not found for action P2pNetworkPubsubAction::IncomingMessageCleanup"
                    );
                    return Ok(());
                };
                client_state.incoming_messages.clear();
                client_state.incoming_messages.shrink_to(0x20);

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
            P2pNetworkPubsubAction::Broadcast { message } => {
                let mut buffer = vec![0; 8];

                if binprot::BinProtWrite::binprot_write(&message, &mut buffer).is_err() {
                    bug_condition!("binprot serialization error");
                    return Ok(());
                }

                let len = buffer.len() - 8;
                buffer[..8].clone_from_slice(&(len as u64).to_le_bytes());

                Self::prepare_to_sign(state_context, buffer)
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
                dispatcher.push(P2pNetworkPubsubEffectfulAction::Sign {
                    author,
                    topic,
                    message,
                });
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

    #[inline(never)]
    fn reduce_incoming_message(
        &mut self,
        peer_id: PeerId,
        message: Message,
        seen_limit: usize,
    ) -> Result<(), String> {
        let topic = self.topics.entry(message.topic.clone()).or_default();

        if let Some(signature) = &message.signature {
            // skip recently seen message
            if !self.seen.contains(signature) {
                self.seen.push_back(signature.clone());
                // keep only last `n` to avoid memory leak
                if self.seen.len() > seen_limit {
                    self.seen.pop_front();
                }
            } else {
                return Ok(());
            }
        }

        if let Some(data) = &message.data {
            if data.len() > 8 {
                let mut slice = &data[8..];
                match gossip::GossipNetMessageV2::binprot_read(&mut slice) {
                    Ok(gossip::GossipNetMessageV2::NewState(block)) => {
                        self.incoming_block = Some((peer_id, block));
                    }
                    Ok(gossip::GossipNetMessageV2::TransactionPoolDiff { message, nonce }) => {
                        let nonce = nonce.as_u32();
                        let txs = message.0.into_iter().map(|tx| (tx, nonce));
                        self.incoming_transactions.extend(txs);
                    }
                    Ok(gossip::GossipNetMessageV2::SnarkPoolDiff { message, nonce }) => {
                        if let v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(work) =
                            message
                        {
                            self.incoming_snarks.push((work.1.into(), nonce.as_u32()));
                        }
                    }
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }
            }
        }

        let message_id = self.mcache.put(message.clone());

        // TODO: this should only happen after the contents have been validated.
        // The only validation that has happened so far is that the message can be parsed.
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
                    state.publish(&message)
                } else {
                    let ctr = state.message.control.get_or_insert_with(Default::default);
                    ctr.ihave.push(pb::ControlIHave {
                        topic_id: Some(message.topic.clone()),
                        message_ids: message_id.clone().into_iter().collect(),
                    })
                }
            });

        Ok(())
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
    /// and message dissemination within the P2P pubsub system.
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
                client_state.buffer.clear();
                client_state.buffer.shrink_to(0x2000);

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
                }

                // TODO: handle other errors
                // TODO: if the error is not a buffer underflow, buffer needs to be cleared.
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
                if let Some(msg) = self.mcache.map.get(msg_id) {
                    if let Some(client) = self.clients.get_mut(peer_id) {
                        client.publish(msg);
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

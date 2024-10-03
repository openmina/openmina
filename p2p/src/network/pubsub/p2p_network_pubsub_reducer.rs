use std::{collections::btree_map::Entry, sync::Arc};

use binprot::BinProtRead;
use mina_p2p_messages::{gossip, v2};
use openmina_core::{block::BlockWithHash, bug_condition, fuzz_maybe, fuzzed_maybe, Substate};
use redux::Dispatcher;

use crate::{
    channels::{snark::P2pChannelsSnarkAction, transaction::P2pChannelsTransactionAction},
    peer::P2pPeerAction,
    Data, P2pConfig, P2pNetworkYamuxAction, PeerId,
};

use super::{
    p2p_network_pubsub_state::P2pNetworkPubsubClientMeshAddingState,
    pb::{self, Message},
    P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubClientTopicState,
    P2pNetworkPubsubEffectfulAction, P2pNetworkPubsubState, TOPIC,
};

impl P2pNetworkPubsubState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, Self>,
        action: redux::ActionWithMeta<&P2pNetworkPubsubAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let pubsub_state = state_context.get_substate_mut()?;

        match action.action() {
            P2pNetworkPubsubAction::NewStream {
                incoming: true,
                peer_id,
                addr,
                protocol,
                ..
            } => {
                let entry = pubsub_state.clients.entry(*peer_id);
                // preserve it
                let outgoing_stream_id = match &entry {
                    Entry::Occupied(v) => v.get().outgoing_stream_id,
                    Entry::Vacant(_) => None,
                };
                let state = entry.or_insert_with(|| P2pNetworkPubsubClientState {
                    protocol: *protocol,
                    addr: *addr,
                    outgoing_stream_id,
                    message: pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: None,
                    },
                    buffer: vec![],
                    incoming_messages: vec![],
                });
                state.protocol = *protocol;
                state.addr = *addr;

                pubsub_state
                    .topics
                    .insert(super::TOPIC.to_owned(), Default::default());

                Ok(())
            }
            P2pNetworkPubsubAction::NewStream {
                incoming: false,
                peer_id,
                stream_id,
                addr,
                protocol,
            } => {
                let state = pubsub_state.clients.entry(*peer_id).or_insert_with(|| {
                    P2pNetworkPubsubClientState {
                        protocol: *protocol,
                        addr: *addr,
                        outgoing_stream_id: Some(*stream_id),
                        message: pb::Rpc {
                            subscriptions: vec![],
                            publish: vec![],
                            control: None,
                        },
                        buffer: vec![],
                        incoming_messages: vec![],
                    }
                });
                state.outgoing_stream_id = Some(*stream_id);
                state.protocol = *protocol;
                state.addr = *addr;

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let config: &P2pConfig = state.substate()?;
                let state: &P2pNetworkPubsubState = state.substate()?;

                let Some(map) = state.topics.get(TOPIC) else {
                    // must have this topic already
                    return Ok(());
                };
                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage {
                    msg: pb::Rpc {
                        subscriptions: vec![pb::rpc::SubOpts {
                            subscribe: Some(true),
                            topic_id: Some(TOPIC.to_owned()),
                        }],
                        publish: vec![],
                        control: None,
                    },
                    peer_id: *peer_id,
                });
                let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                if mesh_size < config.meshsub.outbound_degree_desired {
                    dispatcher.push(P2pNetworkPubsubAction::Graft {
                        peer_id: *peer_id,
                        topic_id: TOPIC.to_owned(),
                    });
                }

                Ok(())
            }
            P2pNetworkPubsubAction::IncomingData {
                peer_id,
                data,
                seen_limit,
                ..
            } => {
                pubsub_state.reduce_incoming_data(peer_id, data)?;

                let dispatcher: &mut Dispatcher<Action, State> = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPubsubEffectfulAction::IncomingData {
                    peer_id: *peer_id,
                    seen_limit: *seen_limit,
                });

                Ok(())
            }
            P2pNetworkPubsubAction::IncomingMessage {
                peer_id,
                message,
                seen_limit,
            } => {
                pubsub_state.reduce_incoming_message(peer_id, message, *seen_limit)?;

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let state: &Self = global_state.substate()?;
                let config: &P2pConfig = global_state.substate()?;
                let peer_id = *peer_id;

                let incoming_block = state.incoming_block.as_ref().cloned();
                let incoming_transactions = state.incoming_transactions.clone();
                let incoming_snarks = state.incoming_snarks.clone();
                let topics = state.topics.clone();

                for (topic_id, map) in topics {
                    let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                    let could_accept = mesh_size < config.meshsub.outbound_degree_high;

                    if !could_accept {
                        if let Some(topic_state) = map.get(&peer_id) {
                            if topic_state.on_mesh() {
                                dispatcher.push(P2pNetworkPubsubAction::Prune { peer_id, topic_id })
                            }
                        }
                    }
                }

                broadcast(dispatcher, global_state)?;
                if let Some((_, block)) = incoming_block {
                    let best_tip = BlockWithHash::try_new(Arc::new(block))?;
                    dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                }
                for (transaction, nonce) in incoming_transactions {
                    dispatcher.push(P2pChannelsTransactionAction::Libp2pReceived {
                        peer_id,
                        transaction: Box::new(transaction),
                        nonce,
                    });
                }
                for (snark, nonce) in incoming_snarks {
                    dispatcher.push(P2pChannelsSnarkAction::Libp2pReceived {
                        peer_id,
                        snark: Box::new(snark),
                        nonce,
                    });
                }
                Ok(())
            }
            // we want to add peer to our mesh
            P2pNetworkPubsubAction::Graft { peer_id, topic_id } => {
                let Some(state) = pubsub_state
                    .topics
                    .get_mut(topic_id)
                    .and_then(|m| m.get_mut(peer_id))
                else {
                    return Ok(());
                };

                state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;

                let dispatcher = state_context.into_dispatcher();
                let msg = pb::Rpc {
                    subscriptions: vec![],
                    publish: vec![],
                    control: Some(pb::ControlMessage {
                        ihave: vec![],
                        iwant: vec![],
                        graft: vec![pb::ControlGraft {
                            topic_id: Some(topic_id.clone()),
                        }],
                        prune: vec![],
                    }),
                };

                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage {
                    msg,
                    peer_id: *peer_id,
                });
                Ok(())
            }
            P2pNetworkPubsubAction::Prune { peer_id, topic_id } => {
                let Some(state) = pubsub_state
                    .topics
                    .get_mut(topic_id)
                    .and_then(|m| m.get_mut(peer_id))
                else {
                    bug_condition!("State not found for action: {action:?}");
                    return Ok(());
                };

                state.mesh = P2pNetworkPubsubClientMeshAddingState::WeRefused;

                let dispatcher = state_context.into_dispatcher();
                let msg = pb::Rpc {
                    subscriptions: vec![],
                    publish: vec![],
                    control: Some(pb::ControlMessage {
                        ihave: vec![],
                        iwant: vec![],
                        graft: vec![],
                        prune: vec![pb::ControlPrune {
                            topic_id: Some(topic_id.clone()),
                            peers: vec![pb::PeerInfo {
                                peer_id: None,
                                signed_peer_record: None,
                            }],
                            backoff: None,
                        }],
                    }),
                };

                dispatcher.push(P2pNetworkPubsubAction::OutgoingMessage {
                    msg,
                    peer_id: *peer_id,
                });
                Ok(())
            }
            P2pNetworkPubsubAction::OutgoingMessage { peer_id, msg } => {
                if let Some(v) = pubsub_state.clients.get_mut(peer_id) {
                    v.message.subscriptions.clear();
                    v.message.publish.clear();
                    v.message.control = None;
                } else {
                    bug_condition!("Invalid state for action: {action:?}");
                }

                let dispatcher = state_context.into_dispatcher();
                if !message_is_empty(msg) {
                    let mut data = vec![];
                    if prost::Message::encode_length_delimited(msg, &mut data).is_err() {
                        dispatcher.push(P2pNetworkPubsubAction::OutgoingMessageError {
                            msg: msg.clone(),
                            peer_id: *peer_id,
                        });
                    } else {
                        dispatcher.push(P2pNetworkPubsubAction::OutgoingData {
                            data: Data::from(data),
                            peer_id: *peer_id,
                        });
                    }
                }
                Ok(())
            }
            P2pNetworkPubsubAction::OutgoingMessageError { .. } => Ok(()),
            P2pNetworkPubsubAction::Broadcast { message } => {
                let mut seqno = pubsub_state.seq;
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let config: &P2pConfig = state.substate()?;
                seqno += config.meshsub.initial_time.as_nanos() as u64;

                let mut buffer = vec![0; 8];

                if binprot::BinProtWrite::binprot_write(message, &mut buffer).is_err() {
                    bug_condition!("binprot serialization error");
                    return Ok(());
                }

                let len = buffer.len() - 8;
                buffer[..8].clone_from_slice(&(len as u64).to_le_bytes());

                dispatcher.push(P2pNetworkPubsubAction::Sign {
                    seqno,
                    author: config.identity_pub_key.peer_id(),
                    data: buffer.into(),
                    topic: super::TOPIC.to_owned(),
                });

                Ok(())
            }
            P2pNetworkPubsubAction::Sign {
                seqno,
                author,
                data,
                topic,
            } => {
                pubsub_state.seq += 1;

                let libp2p_peer_id =
                    libp2p_identity::PeerId::try_from(*author).expect("valid peer_id"); // This can't happen unless something is broken in the configuration
                pubsub_state.to_sign.push_back(pb::Message {
                    from: Some(libp2p_peer_id.to_bytes()),
                    data: Some(data.0.clone().into_vec()),
                    seqno: Some((*seqno).to_be_bytes().to_vec()),
                    topic: topic.clone(),
                    signature: None,
                    key: None,
                });

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPubsubEffectfulAction::Sign {
                    author: *author,
                    topic: topic.clone(),
                });
                Ok(())
            }
            P2pNetworkPubsubAction::SignError { .. } => {
                let _ = pubsub_state.to_sign.pop_front();
                Ok(())
            }
            P2pNetworkPubsubAction::BroadcastSigned { signature } => {
                if let Some(mut message) = pubsub_state.to_sign.pop_front() {
                    message.signature = Some(signature.clone().0.to_vec());
                    pubsub_state
                        .clients
                        .iter_mut()
                        .for_each(|(_, state)| state.message.publish.push(message.clone()));
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                broadcast(dispatcher, state)
            }
            P2pNetworkPubsubAction::OutgoingData { data, peer_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &Self = state.substate()?;
                let mut data = data.clone();
                let Some(state) = state.clients.get(peer_id) else {
                    bug_condition!("Missing state for action: {action:?}");
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

    fn reduce_incoming_message(
        &mut self,
        peer_id: &PeerId,
        message: &Message,
        seen_limit: usize,
    ) -> Result<(), String> {
        self.incoming_transactions.clear();
        self.incoming_snarks.clear();

        let Some(state) = self.clients.get_mut(peer_id) else {
            bug_condition!("State not found for action P2pNetworkPubsubAction::IncomingMessage");
            return Ok(());
        };
        state.incoming_messages.clear();

        let message_id = self.mcache.put(message.clone());

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

        self.clients
            .iter_mut()
            .filter(|(c, _)| {
                // don't send back to who sent this
                **c != *peer_id
            })
            .for_each(|(c, state)| {
                let Some(topic_state) = topic.get(c) else {
                    return;
                };
                if topic_state.on_mesh() {
                    state.message.publish.push(message.clone())
                } else {
                    let ctr = state.message.control.get_or_insert_with(Default::default);
                    ctr.ihave.push(pb::ControlIHave {
                        topic_id: Some(message.topic.clone()),
                        message_ids: message_id.clone().into_iter().collect(),
                    })
                }
            });

        if let Some(data) = &message.data {
            if data.len() > 8 {
                let mut slice = &data[8..];
                match gossip::GossipNetMessageV2::binprot_read(&mut slice) {
                    Ok(gossip::GossipNetMessageV2::NewState(block)) => {
                        self.incoming_block = Some((*peer_id, block));
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

        Ok(())
    }

    fn reduce_incoming_data(&mut self, peer_id: &PeerId, data: &Data) -> Result<(), String> {
        let Some(state) = self.clients.get_mut(peer_id) else {
            // TODO: investigate, cannot reproduce this
            // bug_condition!("State not found for action: P2pNetworkPubsubAction::IncomingData");
            return Ok(());
        };
        let slice = if state.buffer.is_empty() {
            &**data
        } else {
            state.buffer.extend_from_slice(data);
            &state.buffer
        };
        let mut subscriptions = vec![];
        let mut control = pb::ControlMessage::default();
        match <pb::Rpc as prost::Message>::decode_length_delimited(slice) {
            Ok(v) => {
                state.buffer.clear();
                // println!(
                //     "(pubsub) this <- {peer_id}, {:?}, {:?}, {}",
                //     v.subscriptions,
                //     v.control,
                //     v.publish.len()
                // );

                subscriptions.extend_from_slice(&v.subscriptions);
                state.incoming_messages.extend_from_slice(&v.publish);
                if let Some(v) = v.control {
                    control.graft.extend_from_slice(&v.graft);
                    control.prune.extend_from_slice(&v.prune);
                    control.ihave.extend_from_slice(&v.ihave);
                    control.iwant.extend_from_slice(&v.iwant);
                }
            }
            Err(err) => {
                // bad way to check the error, but `prost` doesn't provide better
                if err.to_string().contains("buffer underflow") && state.buffer.is_empty() {
                    state.buffer = data.to_vec();
                }
            }
        }

        for subscription in &subscriptions {
            let topic_id = subscription.topic_id().to_owned();
            let topic = self.topics.entry(topic_id).or_default();

            if subscription.subscribe() {
                if let Entry::Vacant(v) = topic.entry(*peer_id) {
                    v.insert(P2pNetworkPubsubClientTopicState::default());
                }
            } else {
                topic.remove(peer_id);
            }
        }

        for graft in &control.graft {
            if let Some(mesh_state) = self
                .topics
                .get_mut(graft.topic_id())
                .and_then(|m| m.get_mut(peer_id))
            {
                mesh_state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;
            }
        }
        for prune in &control.prune {
            if let Some(mesh_state) = self
                .topics
                .get_mut(prune.topic_id())
                .and_then(|m| m.get_mut(peer_id))
            {
                mesh_state.mesh = P2pNetworkPubsubClientMeshAddingState::TheyRefused;
            }
        }
        for iwant in &control.iwant {
            for msg_id in &iwant.message_ids {
                if let Some(msg) = self.mcache.map.get(msg_id) {
                    if let Some(client) = self.clients.get_mut(peer_id) {
                        client.message.publish.push(msg.clone());
                    }
                }
            }
        }

        for ihave in &control.ihave {
            let message_ids = ihave
                .message_ids
                .iter()
                .filter(|msg_id| !self.mcache.map.contains_key(*msg_id))
                .cloned()
                .collect();
            if let Some(client) = self.clients.get_mut(peer_id) {
                let ctr = client.message.control.get_or_insert_with(Default::default);
                ctr.iwant.push(pb::ControlIWant { message_ids })
            }
        }
        Ok(())
    }
}

fn message_is_empty(msg: &pb::Rpc) -> bool {
    msg.subscriptions.is_empty() && msg.publish.is_empty() && msg.control.is_none()
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

    state
        .clients
        .iter()
        .filter(|(_, state)| !message_is_empty(&state.message))
        .map(|(peer_id, state)| P2pNetworkPubsubAction::OutgoingMessage {
            msg: state.message.clone(),
            peer_id: *peer_id,
        })
        .for_each(|action| dispatcher.push(action));

    Ok(())
}

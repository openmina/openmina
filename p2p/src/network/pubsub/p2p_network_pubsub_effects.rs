use std::sync::Arc;

use openmina_core::{block::BlockWithHash, fuzz_maybe, fuzzed_maybe};

use crate::{
    channels::{snark::P2pChannelsSnarkAction, transaction::P2pChannelsTransactionAction},
    peer::P2pPeerAction,
    P2pCryptoService, P2pNetworkYamuxAction,
};

use super::{pb, P2pNetworkPubsubAction, TOPIC};

fn message_is_empty(msg: &pb::Rpc) -> bool {
    msg.subscriptions.is_empty() && msg.publish.is_empty() && msg.control.is_none()
}

impl P2pNetworkPubsubAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pCryptoService,
    {
        let state = &store.state().network.scheduler.broadcast_state;
        let config = &store.state().config;

        // let this = config.identity_pub_key.peer_id();

        match self {
            P2pNetworkPubsubAction::NewStream {
                peer_id, incoming, ..
            } => {
                // println!("(pubsub) {this} new stream {peer_id} {incoming}");
                if !incoming {
                    let subscrption = {
                        let msg = pb::Rpc {
                            subscriptions: vec![pb::rpc::SubOpts {
                                subscribe: Some(true),
                                topic_id: Some(TOPIC.to_owned()),
                            }],
                            publish: vec![],
                            control: None,
                        };
                        Some(P2pNetworkPubsubAction::OutgoingMessage { msg, peer_id })
                    };
                    let Some(map) = state.topics.get(TOPIC) else {
                        // must have this topic already
                        return;
                    };
                    let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                    let graft = if mesh_size < config.meshsub.outbound_degree_desired {
                        Some(P2pNetworkPubsubAction::Graft {
                            peer_id,
                            topic_id: TOPIC.to_owned(),
                        })
                    } else {
                        None
                    };
                    for action in subscrption.into_iter().chain(graft) {
                        store.dispatch(action);
                    }
                }
            }
            P2pNetworkPubsubAction::Graft { peer_id, topic_id } => {
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

                store.dispatch(P2pNetworkPubsubAction::OutgoingMessage { msg, peer_id });
            }
            P2pNetworkPubsubAction::Prune { peer_id, topic_id } => {
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

                store.dispatch(P2pNetworkPubsubAction::OutgoingMessage { msg, peer_id });
            }
            P2pNetworkPubsubAction::Broadcast { message } => {
                // println!("(pubsub) {this} broadcast");
                let mut buffer = vec![0; 8];
                if binprot::BinProtWrite::binprot_write(&message, &mut buffer).is_err() {
                    return;
                }
                let len = buffer.len() - 8;
                buffer[..8].clone_from_slice(&(len as u64).to_le_bytes());

                store.dispatch(P2pNetworkPubsubAction::Sign {
                    seqno: state.seq + config.meshsub.initial_time.as_nanos() as u64,
                    author: config.identity_pub_key.peer_id(),
                    data: buffer.into(),
                    topic: TOPIC.to_owned(),
                });
            }
            P2pNetworkPubsubAction::Sign { author, topic, .. } => {
                if let Some(to_sign) = state.to_sign.front() {
                    let mut publication = vec![];
                    if prost::Message::encode(to_sign, &mut publication).is_err() {
                        store.dispatch(P2pNetworkPubsubAction::SignError { author, topic });
                    } else {
                        let signature = store.service().sign_publication(&publication).into();
                        store.dispatch(P2pNetworkPubsubAction::BroadcastSigned { signature });
                    }
                }
            }
            P2pNetworkPubsubAction::BroadcastSigned { .. } => broadcast(store),
            P2pNetworkPubsubAction::IncomingData {
                peer_id,
                seen_limit,
                ..
            } => {
                let Some(state) = state.clients.get(&peer_id) else {
                    return;
                };
                let messages = state.incoming_messages.clone();

                for mut message in messages {
                    if let (Some(signature), Some(from)) =
                        (message.signature.take(), message.from.clone())
                    {
                        message.key = None;
                        let mut data = vec![];
                        if prost::Message::encode(&message, &mut data).is_err() {
                            continue;
                        } else if !store
                            .service()
                            .verify_publication(&from[2..], &data, &signature)
                        {
                            continue;
                        }
                    } else {
                        // the message doesn't contain signature or it doesn't contain verifying key
                        continue;
                    }
                    store.dispatch(P2pNetworkPubsubAction::IncomingMessage {
                        peer_id,
                        message,
                        seen_limit,
                    });
                }
            }
            P2pNetworkPubsubAction::IncomingMessage { peer_id, .. } => {
                // println!("(pubsub) {this} <- {peer_id}");

                let incoming_block = state.incoming_block.as_ref().cloned();
                let incoming_transactions = state.incoming_transactions.clone();
                let incoming_snarks = state.incoming_snarks.clone();
                let topics = state.topics.clone();

                let mut prune_at_topics = vec![];
                for (topic_id, map) in topics {
                    let mesh_size = map.values().filter(|s| s.on_mesh()).count();
                    let could_accept = mesh_size < config.meshsub.outbound_degree_high;

                    if !could_accept {
                        if let Some(topic_state) = map.get(&peer_id) {
                            if topic_state.on_mesh() {
                                prune_at_topics.push(topic_id);
                            }
                        }
                    }
                }

                for topic_id in prune_at_topics {
                    store.dispatch(Self::Prune { peer_id, topic_id });
                }

                broadcast(store);
                if let Some((_, block)) = incoming_block {
                    let best_tip = BlockWithHash::new(Arc::new(block));
                    store.dispatch(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                }
                for (transaction, nonce) in incoming_transactions {
                    store.dispatch(P2pChannelsTransactionAction::Libp2pReceived {
                        peer_id,
                        transaction: Box::new(transaction),
                        nonce,
                    });
                }
                for (snark, nonce) in incoming_snarks {
                    store.dispatch(P2pChannelsSnarkAction::Libp2pReceived {
                        peer_id,
                        snark: Box::new(snark),
                        nonce,
                    });
                }
            }
            P2pNetworkPubsubAction::OutgoingMessage { msg, peer_id } => {
                if !message_is_empty(&msg) {
                    // println!(
                    //     "(pubsub) {this} -> {peer_id}, {:?}, {:?}, {}",
                    //     msg.subscriptions,
                    //     msg.control,
                    //     msg.publish.len()
                    // );
                    // for ele in &msg.publish {
                    //     let id = super::p2p_network_pubsub_state::compute_message_id(ele);
                    //     println!("{}", std::str::from_utf8(&id).unwrap());
                    // }
                    let mut data = vec![];
                    if prost::Message::encode_length_delimited(&msg, &mut data).is_err() {
                        store.dispatch(P2pNetworkPubsubAction::OutgoingMessageError {
                            msg,
                            peer_id,
                        });
                    } else {
                        store.dispatch(P2pNetworkPubsubAction::OutgoingData {
                            data: data.into(),
                            peer_id,
                        });
                    }
                }
            }
            P2pNetworkPubsubAction::OutgoingMessageError { .. } => {}
            P2pNetworkPubsubAction::OutgoingData { mut data, peer_id } => {
                let Some(state) = store
                    .state()
                    .network
                    .scheduler
                    .broadcast_state
                    .clients
                    .get(&peer_id)
                else {
                    return;
                };
                fuzz_maybe!(&mut data, crate::fuzzer::mutate_pubsub);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                if let Some(stream_id) = state.outgoing_stream_id.as_ref().copied() {
                    store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                        addr: state.addr,
                        stream_id,
                        data,
                        flags,
                    });
                }
            }
            P2pNetworkPubsubAction::SignError { .. } => (),
        }
    }
}

fn broadcast<Store, S>(store: &mut Store)
where
    Store: crate::P2pStore<S>,
{
    let state = &store.state().network.scheduler.broadcast_state;
    let broadcast = state
        .clients
        .iter()
        .filter(|(_, state)| !message_is_empty(&state.message))
        .map(|(peer_id, state)| P2pNetworkPubsubAction::OutgoingMessage {
            msg: state.message.clone(),
            peer_id: *peer_id,
        })
        .collect::<Vec<_>>();
    for action in broadcast {
        store.dispatch(action);
    }
}

use std::sync::Arc;

use openmina_core::block::BlockWithHash;
use snark::user_command_verify::SnarkUserCommandVerifyAction;

use crate::{
    channels::snark::P2pChannelsSnarkAction, peer::P2pPeerAction, P2pCryptoService,
    P2pNetworkYamuxAction,
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
        match self {
            Self::NewStream {
                peer_id, incoming, ..
            } => {
                if !incoming {
                    let msg = pb::Rpc {
                        subscriptions: vec![pb::rpc::SubOpts {
                            subscribe: Some(true),
                            topic_id: Some(TOPIC.to_owned()),
                        }],
                        publish: vec![],
                        control: None,
                    };
                    store.dispatch(Self::OutgoingMessage { msg, peer_id });
                    let msg = pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: Some(pb::ControlMessage {
                            ihave: vec![],
                            iwant: vec![],
                            graft: vec![pb::ControlGraft {
                                topic_id: Some(TOPIC.to_owned()),
                            }],
                            prune: vec![],
                        }),
                    };
                    store.dispatch(Self::OutgoingMessage { msg, peer_id });
                }
            }
            Self::Broadcast { message } => {
                let mut buffer = vec![0; 8];
                binprot::BinProtWrite::binprot_write(&message, &mut buffer).expect("msg");
                let len = buffer.len() - 8;
                buffer[..8].clone_from_slice(&(len as u64).to_le_bytes());

                store.dispatch(Self::Sign {
                    seqno: state.seq + store.state().config.initial_time.as_nanos() as u64,
                    author: store.state().config.identity_pub_key.peer_id(),
                    data: buffer.into(),
                    topic: TOPIC.to_owned(),
                });
            }
            Self::Sign { .. } => {
                if let Some(to_sign) = state.to_sign.front() {
                    let mut publication = vec![];
                    prost::Message::encode(to_sign, &mut publication).unwrap();
                    let signature = store.service().sign_publication(&publication).into();
                    store.dispatch(Self::BroadcastSigned { signature });
                }
            }
            Self::BroadcastSigned { .. } => broadcast(store),
            Self::IncomingData { peer_id, .. } => {
                let incoming_block = state.incoming_block.as_ref().cloned();
                let incoming_snarks = state.incoming_snarks.clone();
                let incoming_txs = state.incoming_transactions.clone();

                broadcast(store);
                if let Some((_, block)) = incoming_block {
                    let best_tip = BlockWithHash::new(Arc::new(block));
                    store.dispatch(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                }
                for (snark, nonce) in incoming_snarks {
                    store.dispatch(P2pChannelsSnarkAction::Libp2pReceived {
                        peer_id,
                        snark,
                        nonce,
                    });
                }
                if let Some((txs, nonce)) = incoming_txs {
                    store.dispatch(SnarkUserCommandVerifyAction::Receive {
                        commands: txs,
                        nonce,
                    });
                };
            }
            Self::OutgoingMessage { msg, peer_id } => {
                if !message_is_empty(&msg) {
                    let mut data = vec![];
                    if prost::Message::encode_length_delimited(&msg, &mut data).is_ok() {
                        store.dispatch(Self::OutgoingData {
                            data: data.clone().into(),
                            peer_id,
                        });
                    }
                }
            }
            Self::OutgoingData { data, peer_id } => {
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
                if let Some(stream_id) = state.outgoing_stream_id.as_ref().copied() {
                    store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                        addr: state.addr,
                        stream_id,
                        data,
                        fin: false,
                    });
                }
            }
        }
    }
}

pub fn broadcast<Store, S>(store: &mut Store)
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

use crate::{P2pCryptoService, P2pNetworkYamuxAction};

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
            Self::Broadcast { data, topic } => {
                let author = store.state().config.identity_pub_key.peer_id();
                store.dispatch(Self::Sign {
                    seqno: state.seq + store.state().config.initial_time.as_nanos() as u64,
                    author,
                    data,
                    topic,
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
            Self::IncomingData { .. } | Self::BroadcastSigned { .. } => {
                let broadcast = state
                    .clients
                    .iter()
                    .filter(|(_, state)| !message_is_empty(&state.message))
                    .map(|(peer_id, state)| Self::OutgoingMessage {
                        msg: state.message.clone(),
                        peer_id: *peer_id,
                    })
                    .collect::<Vec<_>>();
                for action in broadcast {
                    store.dispatch(action);
                }
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

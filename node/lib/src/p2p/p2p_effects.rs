use crate::rpc::{RpcP2pConnectionOutgoingErrorAction, RpcP2pConnectionOutgoingSuccessAction};
use crate::{Service, Store};

use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::P2pConnectionAction;
use super::pubsub::P2pPubsubAction;
use super::{P2pAction, P2pActionWithMeta};

pub fn p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => match action {
                P2pConnectionOutgoingAction::Init(action) => {
                    action.effects(meta, store);
                }
                P2pConnectionOutgoingAction::Pending(_) => {
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Error(content) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&content.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingErrorAction {
                            rpc_id,
                            error: content.error.clone(),
                        });
                    }

                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(content) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&content.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingSuccessAction { rpc_id });
                    }
                    // action.effects(&meta, store);
                }
            },
        },
        P2pAction::Pubsub(action) => match action {
            P2pPubsubAction::MessagePublish(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::BytesPublish(action) => {
                let rpc_id = action.rpc_id;
                action.effects(&meta, store);
                if let Some(rpc_id) = rpc_id {
                    // TODO(binier)
                    let _ = store.service.respond_p2p_publish(rpc_id, Ok(()));
                }
            }
            P2pPubsubAction::BytesReceived(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::MessageReceived(_) => {}
        },
    }
}

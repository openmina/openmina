use crate::action::CheckTimeoutsAction;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingErrorAction, P2pConnectionOutgoingSuccessAction,
};
use crate::p2p::disconnection::P2pDisconnectionFinishAction;
use crate::p2p::pubsub::P2pPubsubBytesReceivedAction;
use crate::p2p::rpc::outgoing::{P2pRpcOutgoingErrorAction, P2pRpcOutgoingReceivedAction};
use crate::rpc::{
    RpcGlobalStateGetAction, RpcP2pConnectionOutgoingInitAction, RpcP2pPubsubMessagePublishAction,
    RpcRequest, RpcWatchedAccountsAddAction, RpcWatchedAccountsGetAction,
};
use crate::snark::block_verify::{SnarkBlockVerifyErrorAction, SnarkBlockVerifySuccessAction};
use crate::{Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, EventSourceNewEventAction,
    P2pConnectionEvent, P2pEvent, P2pPubsubEvent, P2pRpcEvent, SnarkEvent,
};

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, _) = action.split();
    match action {
        EventSourceAction::ProcessEvents(_) => {
            // process max 1024 events at a time.
            for _ in 0..1024 {
                match store.service.next_event() {
                    Some(event) => {
                        store.dispatch(EventSourceNewEventAction { event });
                    }
                    None => break,
                }
            }
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::NewEvent(content) => match content.event {
            Event::P2p(e) => match e {
                P2pEvent::Connection(e) => match e {
                    P2pConnectionEvent::OutgoingInit(peer_id, result) => match result {
                        Err(error) => {
                            store.dispatch(P2pConnectionOutgoingErrorAction { peer_id, error });
                        }
                        Ok(_) => {
                            store.dispatch(P2pConnectionOutgoingSuccessAction { peer_id });
                        }
                    },
                    P2pConnectionEvent::Closed(peer_id) => {
                        store.dispatch(P2pDisconnectionFinishAction { peer_id });
                    }
                },
                P2pEvent::Pubsub(e) => match e {
                    P2pPubsubEvent::BytesReceived {
                        author,
                        sender,
                        topic,
                        bytes,
                    } => {
                        store.dispatch(P2pPubsubBytesReceivedAction {
                            author,
                            sender,
                            topic,
                            bytes,
                        });
                    }
                },
                P2pEvent::Rpc(e) => match e {
                    P2pRpcEvent::OutgoingError(peer_id, rpc_id, error) => {
                        store.dispatch(P2pRpcOutgoingErrorAction {
                            peer_id,
                            rpc_id,
                            error,
                        });
                    }
                    P2pRpcEvent::OutgoingResponse(peer_id, rpc_id, response) => {
                        store.dispatch(P2pRpcOutgoingReceivedAction {
                            peer_id,
                            rpc_id,
                            response,
                        });
                    }
                },
            },
            Event::Snark(event) => match event {
                SnarkEvent::BlockVerify(req_id, result) => match result {
                    Err(error) => {
                        store.dispatch(SnarkBlockVerifyErrorAction { req_id, error });
                    }
                    Ok(()) => {
                        store.dispatch(SnarkBlockVerifySuccessAction { req_id });
                    }
                },
            },
            Event::Rpc(rpc_id, e) => match e {
                RpcRequest::GetState => {
                    store.dispatch(RpcGlobalStateGetAction { rpc_id });
                }
                RpcRequest::P2pConnectionOutgoing(opts) => {
                    store.dispatch(RpcP2pConnectionOutgoingInitAction { rpc_id, opts });
                }
                RpcRequest::P2pPubsubPublish(topic, message) => {
                    store.dispatch(RpcP2pPubsubMessagePublishAction {
                        rpc_id,
                        topic,
                        message,
                    });
                }
                RpcRequest::WatchedAccountsAdd(pub_key) => {
                    store.dispatch(RpcWatchedAccountsAddAction { rpc_id, pub_key });
                }
                RpcRequest::WatchedAccountsGet(pub_key) => {
                    store.dispatch(RpcWatchedAccountsGetAction { rpc_id, pub_key });
                }
            },
        },
        EventSourceAction::WaitTimeout(_) => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents(_) => {}
    }
}

use crate::action::CheckTimeoutsAction;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingErrorAction, P2pConnectionOutgoingSuccessAction,
};
use crate::rpc::{RpcGlobalStateGetAction, RpcRequest};
use crate::{Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, EventSourceNewEventAction,
    P2pConnectionEvent, P2pEvent,
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
                },
            },
            Event::Rpc(rpc_id, e) => match e {
                RpcRequest::GetState => {
                    store.dispatch(RpcGlobalStateGetAction { rpc_id });
                }
            },
        },
        EventSourceAction::WaitTimeout(_) => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents(_) => {}
    }
}

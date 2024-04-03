use openmina_core::{action_debug, action_warn, log::ActionEvent};
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{P2pListenerId, P2pState};

pub type P2pListenActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pListenAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenAction {
    New {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    Expired {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    Error {
        listener_id: P2pListenerId,
        error: String,
    },
    Closed {
        listener_id: P2pListenerId,
        error: Option<String>,
    },
}

impl EnablingCondition<P2pState> for P2pListenAction {}

impl ActionEvent for P2pListenAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            P2pListenAction::New { listener_id, addr } => action_debug!(
                context,
                listener_id = debug(listener_id),
                addr = display(addr)
            ),
            P2pListenAction::Expired { listener_id, addr } => action_debug!(
                context,
                listener_id = debug(listener_id),
                addr = display(addr)
            ),
            P2pListenAction::Error { listener_id, error } => {
                action_warn!(context, listener_id = debug(listener_id), error)
            }
            P2pListenAction::Closed { listener_id, error } => {
                action_debug!(context, listener_id = debug(listener_id), error)
            }
        }
    }
}

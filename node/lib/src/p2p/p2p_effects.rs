use crate::{Service, Store};

use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::P2pConnectionAction;
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
                P2pConnectionOutgoingAction::Error(_) => {
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(_) => {
                    // action.effects(&meta, store);
                }
            },
        },
    }
}

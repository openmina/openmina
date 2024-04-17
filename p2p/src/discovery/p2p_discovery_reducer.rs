use crate::P2pState;

use super::{P2pDiscoveryAction, P2pDiscoveryActionWithMetaRef};

pub fn p2p_discovery_reducer(_state: &mut P2pState, action: P2pDiscoveryActionWithMetaRef) {
    let (action, _meta) = action.split();

    match action {
        P2pDiscoveryAction::Init { .. } => {}
        P2pDiscoveryAction::Success { .. } => {
            // TODO: update timestamp
        }
    }
}

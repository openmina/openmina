use openmina_core::log::error;
use redux::ActionWithMeta;

use crate::P2pPeerState;

use super::{webrtc::p2p_connection_webrtc_reducer, P2pConnectionAction};

pub fn p2p_connection_reducer(
    state: &mut P2pPeerState,
    action: ActionWithMeta<&'_ P2pConnectionAction>,
) {
    let (action, meta) = action.split();
    // TODO(akoptelov): decide if method or function style reducer should be used
    match action {
        P2pConnectionAction::LibP2p(action) => {
            let P2pPeerState::Libp2p(libp2p_state) = state else {
                error!(meta.time(); "invalid peer state {state:?}");
                return;
            };
            libp2p_state.reducer(meta.with_action(action));
        }
        P2pConnectionAction::WebRTC(action) => {
            let P2pPeerState::WebRTC(webrtc_state) = state else {
                error!(meta.time(); "invalid peer state {state:?}");
                return;
            };

            p2p_connection_webrtc_reducer(webrtc_state, meta.with_action(action));
        }
    }
}

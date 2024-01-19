use redux::ActionWithMeta;

use super::{
    P2pConnectionLibP2pIncomingAction, P2pConnectionLibP2pIncomingState,
    P2pConnectionLibP2pIncomingSuccessState,
};

impl P2pConnectionLibP2pIncomingState {
    pub fn reducer(&mut self, action: ActionWithMeta<&'_ P2pConnectionLibP2pIncomingAction>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionLibP2pIncomingAction::Success(action) => {
                *self = Self::Success(P2pConnectionLibP2pIncomingSuccessState {
                    peer_id: action.peer_id,
                    time: meta.time(),
                })
            }
        }
    }
}

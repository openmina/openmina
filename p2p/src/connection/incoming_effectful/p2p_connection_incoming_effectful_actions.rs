use crate::{
    connection::{incoming::P2pConnectionIncomingInitOpts, P2pConnectionEffectfulAction},
    webrtc, P2pAction, P2pState, PeerId,
};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(debug(opts), display(peer_id), display(error)))]
pub enum P2pConnectionIncomingEffectfulAction {
    /// Incoming connection is initialized.
    Init { opts: P2pConnectionIncomingInitOpts },
    AnswerReady {
        peer_id: PeerId,
        answer: Box<webrtc::Answer>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingEffectfulAction {
    fn is_enabled(&self, _: &P2pState, _: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pConnectionIncomingEffectfulAction> for P2pAction {
    fn from(a: P2pConnectionIncomingEffectfulAction) -> Self {
        Self::ConnectionEffectful(P2pConnectionEffectfulAction::Incoming(a))
    }
}

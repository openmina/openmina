use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionEffectfulAction},
    webrtc, P2pState, PeerId,
};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(opts), display(peer_id), display(error)))]
pub enum P2pConnectionOutgoingEffectfulAction {
    /// Initialize connection to a random peer.
    #[action_event(level = trace)]
    RandomInit,
    /// Initialize connection to a new peer.
    #[action_event(level = info)]
    Init {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferReady {
        peer_id: PeerId,
        offer: Box<webrtc::Offer>,
    },
    AnswerRecvSuccess {
        peer_id: PeerId,
        answer: Box<webrtc::Answer>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingEffectfulAction {
    fn is_enabled(&self, state: &P2pState, _: redux::Timestamp) -> bool {
        match self {
            P2pConnectionOutgoingEffectfulAction::RandomInit => {
                !state.already_has_min_peers() && state.disconnected_peers().next().is_some()
            }
            _ => true,
        }
    }
}

impl From<P2pConnectionOutgoingEffectfulAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingEffectfulAction) -> Self {
        Self::ConnectionEffectful(P2pConnectionEffectfulAction::Outgoing(a))
    }
}

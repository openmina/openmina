use openmina_core::{snark::SnarkJobCommitment, ActionEvent};
use serde::{Deserialize, Serialize};

use crate::{channels::P2pChannelsEffectfulAction, P2pState, PeerId};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSnarkJobCommitmentEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        commitments: Vec<SnarkJobCommitment>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsSnarkJobCommitmentEffectfulAction> for crate::P2pAction {
    fn from(action: P2pChannelsSnarkJobCommitmentEffectfulAction) -> crate::P2pAction {
        crate::P2pAction::ChannelsEffectful(P2pChannelsEffectfulAction::SnarkJobCommitment(action))
    }
}

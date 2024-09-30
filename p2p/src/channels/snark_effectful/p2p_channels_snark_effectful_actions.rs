use crate::{channels::P2pChannelsEffectfulAction, P2pState, PeerId};
use openmina_core::{snark::SnarkInfo, ActionEvent};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSnarkEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        snarks: Vec<SnarkInfo>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsSnarkEffectfulAction> for crate::P2pAction {
    fn from(action: P2pChannelsSnarkEffectfulAction) -> crate::P2pAction {
        crate::P2pAction::ChannelsEffectful(P2pChannelsEffectfulAction::Snark(action))
    }
}

use openmina_core::{block::ArcBlockWithHash, ActionEvent};
use serde::{Deserialize, Serialize};

use crate::{channels::P2pChannelsEffectfulAction, P2pState, PeerId};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id), best_tip = display(&best_tip.hash)))]
pub enum P2pChannelsBestTipEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
    },
    ResponseSend {
        peer_id: PeerId,
        best_tip: ArcBlockWithHash,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsBestTipEffectfulAction> for crate::P2pAction {
    fn from(action: P2pChannelsBestTipEffectfulAction) -> crate::P2pAction {
        crate::P2pAction::ChannelsEffectful(P2pChannelsEffectfulAction::BestTip(action))
    }
}

use openmina_core::transaction::TransactionInfo;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{channels::P2pChannelsEffectfulAction, P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsTransactionEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        transactions: Vec<TransactionInfo>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsTransactionEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsTransactionEffectfulAction> for crate::P2pEffectfulAction {
    fn from(action: P2pChannelsTransactionEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Channels(P2pChannelsEffectfulAction::Transaction(action))
    }
}

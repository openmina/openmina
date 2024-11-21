use crate::ledger::{
    read::{LedgerReadIdType, LedgerReadInitCallback, LedgerReadRequest},
    write::LedgerWriteRequest,
};
use openmina_core::requests::RequestId;
use redux::Callback;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerEffectfulAction {
    WriteInit {
        request: LedgerWriteRequest,
        on_init: Callback<LedgerWriteRequest>,
    },
    WriteSuccess,
    ReadInit {
        request: LedgerReadRequest,
        callback: LedgerReadInitCallback,
        id: RequestId<LedgerReadIdType>,
    },
}

impl redux::EnablingCondition<crate::State> for LedgerEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}

use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{external_snark_worker::SnarkWorkSpec, State};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
pub enum ExternalSnarkWorkerEffectfulAction {
    Start {
        public_key: NonZeroCurvePoint,
        fee: CurrencyFeeStableV1,
    },
    Kill,
    SubmitWork {
        spec: Box<SnarkWorkSpec>,
    },
    CancelWork,
}

impl EnablingCondition<State> for ExternalSnarkWorkerEffectfulAction {
    fn is_enabled(&self, _state: &State, _time: redux::Timestamp) -> bool {
        true
    }
}

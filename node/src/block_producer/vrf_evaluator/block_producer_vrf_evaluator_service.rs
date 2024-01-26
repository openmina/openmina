use std::collections::BTreeMap;

use ledger::AccountIndex;
use mina_p2p_messages::v2::LedgerHash;

use crate::account::AccountPublicKey;

use super::{DelegatorTable, VrfEvaluatorInput};

pub trait BlockProducerVrfEvaluatorService: redux::Service {
    fn evaluate(&mut self, data: VrfEvaluatorInput);
}

pub trait BlockProducerVrfEvaluatorLedgerService: redux::Service {
    fn get_producer_and_delegates(
        &mut self,
        ledger_hash: LedgerHash,
        producer: AccountPublicKey,
    ) -> DelegatorTable;
}

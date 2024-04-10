use mina_p2p_messages::v2::LedgerHash;

use openmina_node_account::AccountPublicKey;

use super::{DelegatorTable, VrfEvaluatorInput};

// TODO(adonagy): Use just one trait as VrfEvaluatorService
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

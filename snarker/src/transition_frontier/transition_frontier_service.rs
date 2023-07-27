use std::collections::BTreeSet;

use ledger::scan_state::scan_state::{transaction_snark::OneOrTwo, AvailableJobMessage};
use mina_p2p_messages::v2::LedgerHash;
use shared::block::ArcBlockWithHash;

pub trait TransitionFrontierService: redux::Service {
    fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String>;
    fn commit(
        &mut self,
        ledgers_to_keep: BTreeSet<LedgerHash>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> Vec<OneOrTwo<AvailableJobMessage>>;
}

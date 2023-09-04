mod watched_accounts_state;
pub use watched_accounts_state::*;

mod watched_accounts_actions;
pub use watched_accounts_actions::*;

mod watched_accounts_reducer;
pub use watched_accounts_reducer::*;

mod watched_accounts_effects;
pub use watched_accounts_effects::*;

use mina_p2p_messages::v2::{
    NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1, StagedLedgerDiffDiffDiffStableV2,
    StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
};

pub fn is_transaction_affecting_account(
    pub_key: &NonZeroCurvePoint,
    tx: &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
) -> bool {
    use ledger::scan_state::transaction_logic::UserCommand;
    UserCommand::from(&tx.data)
        .accounts_referenced()
        .iter()
        .map(|v| {
            NonZeroCurvePoint::from(NonZeroCurvePointUncompressedStableV1 {
                x: v.public_key.x.into(),
                is_odd: v.public_key.is_odd,
            })
        })
        .any(|referenced_pub_key| &referenced_pub_key == pub_key)
}

pub fn account_relevant_transactions_in_diff_iter<'a>(
    pub_key: &'a NonZeroCurvePoint,
    diff: &'a StagedLedgerDiffDiffDiffStableV2,
) -> impl 'a + Iterator<Item = Transaction> {
    let iter_0 = diff.0.commands.iter();
    let iter_1: Box<
        dyn Iterator<Item = &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    > = match &diff.1 {
        Some(v) => Box::new(v.commands.iter()),
        None => Box::new(std::iter::empty()),
    };
    iter_0
        .chain(iter_1)
        .filter(|tx| is_transaction_affecting_account(pub_key, tx))
        .map(|tx| Transaction {
            hash: tx.data.hash().ok(),
            data: tx.data.clone(),
            status: tx.status.clone(),
        })
}

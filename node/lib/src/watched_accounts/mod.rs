mod watched_accounts_state;
pub use watched_accounts_state::*;

mod watched_accounts_actions;
pub use watched_accounts_actions::*;

mod watched_accounts_reducer;
pub use watched_accounts_reducer::*;

mod watched_accounts_effects;
pub use watched_accounts_effects::*;

use mina_p2p_messages::v2::{
    MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseStakeDelegationStableV1,
    MinaBaseUserCommandStableV2, NonZeroCurvePoint, StagedLedgerDiffDiffDiffStableV2,
    StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
};

pub fn is_transaction_affecting_account(
    pub_key: &NonZeroCurvePoint,
    tx: &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
) -> bool {
    match &tx.data {
        MinaBaseUserCommandStableV2::SignedCommand(v) => match &v.payload.body {
            MinaBaseSignedCommandPayloadBodyStableV2::Payment(v) => {
                &v.source_pk == pub_key || &v.receiver_pk == pub_key
            }
            MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(v) => match v {
                MinaBaseStakeDelegationStableV1::SetDelegate {
                    delegator,
                    new_delegate,
                } => delegator == pub_key || new_delegate == pub_key,
            },
        },
        MinaBaseUserCommandStableV2::ZkappCommand(v) => v
            .account_updates
            .iter()
            .any(|v| &v.elt.account_update.body.public_key == pub_key),
    }
}

pub fn account_relevant_transactions_in_diff_iter<'a>(
    pub_key: &'a NonZeroCurvePoint,
    diff: &'a StagedLedgerDiffDiffDiffStableV2,
) -> impl 'a + Iterator<Item = StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B> {
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
        .cloned()
}

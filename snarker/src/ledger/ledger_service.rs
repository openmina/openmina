use std::{collections::BTreeMap, sync::Arc};

use ledger::{
    scan_state::{
        currency::{Amount, Fee},
        scan_state::ConstraintConstants,
        transaction_logic::local_state::LocalState,
    },
    staged_ledger::staged_ledger::StagedLedger,
    verifier::Verifier,
    AccountIndex, BaseLedger, Mask, TreeVersion,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    hash::MinaHash,
    v2::{LedgerHash, MinaBaseAccountBinableArgStableV2},
};

use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerService;

use super::{LedgerAddress, LEDGER_DEPTH};

const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: Amount::from_u64(720000000000),
    supercharged_coinbase_factor: 2,
    account_creation_fee: Fee::from_u64(1000000000),
    fork: None,
};

fn ledger_hash(depth: usize, left: Fp, right: Fp) -> Fp {
    let height = LEDGER_DEPTH - depth - 1;
    ledger::V2::hash_node(height, left, right)
}

#[derive(Default)]
pub struct LedgerCtx {
    sync: Option<LedgerSyncState>,
}

struct LedgerSyncState {
    root_ledger: Mask,
    root_staged_ledger: Option<StagedLedger>,
}

impl LedgerCtx {
    pub fn new() -> Self {
        Default::default()
    }
}

pub trait LedgerService: redux::Service {
    fn ctx(&self) -> &LedgerCtx;
    fn ctx_mut(&mut self) -> &mut LedgerCtx;
}

impl<T: LedgerService> TransitionFrontierSyncLedgerService for T {
    fn root_set(&mut self, hash: LedgerHash) {
        let mut root_ledger = Mask::create(LEDGER_DEPTH);
        root_ledger.set_cached_hash_unchecked(&LedgerAddress::root(), hash.0.to_field());

        self.ctx_mut().sync = Some(LedgerSyncState {
            root_ledger,
            root_staged_ledger: None,
        });
    }

    fn hashes_set(
        &mut self,
        parent: &LedgerAddress,
        (left, right): (LedgerHash, LedgerHash),
    ) -> Result<(), ()> {
        let (left, right) = (left.0.to_field(), right.0.to_field());
        let hash = ledger_hash(parent.length(), left, right);

        let Some(sync) = self.ctx_mut().sync.as_mut() else {
            return Err(());
        };
        let mask = &mut sync.root_ledger;

        if hash != mask.get_inner_hash_at_addr(parent.clone())? {
            return Err(());
        }

        mask.set_cached_hash_unchecked(&parent.child_left(), left);
        mask.set_cached_hash_unchecked(&parent.child_right(), right);

        Ok(())
    }

    fn accounts_set(
        &mut self,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()> {
        // TODO(binier): validate hashes
        let mut addr = parent.clone();
        let first_addr = loop {
            if addr.length() == LEDGER_DEPTH {
                break addr;
            }
            addr = addr.child_left();
        };
        let Some(sync) = self.ctx_mut().sync.as_mut() else {
            return Err(());
        };
        let mask = &mut sync.root_ledger;

        let first_index = first_addr.to_index();
        accounts
            .into_iter()
            .enumerate()
            .try_for_each(|(index, account)| {
                let index = AccountIndex(first_index.0 + index as u64);
                mask.set_at_index(index, account.into())
            })?;

        Ok(())
    }

    fn staged_ledger_reconstruct(
        &mut self,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    ) -> Result<(), String> {
        let Some(sync) = self.ctx_mut().sync.as_mut() else {
            return Err("unexpected state".to_owned());
        };
        let mask = &mut sync.root_ledger;
        // TODO(binier): TMP. Remove for prod version.
        mask.validate_inner_hashes()
            .map_err(|_| "downloaded hash and recalculated mismatch".to_owned())?;

        let states = parts
            .needed_blocks
            .iter()
            .map(|state| (state.hash(), state.clone()))
            .collect::<BTreeMap<_, _>>();

        sync.root_staged_ledger = Some(
            StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
                (),
                &CONSTRAINT_CONSTANTS,
                Verifier,
                (&parts.scan_state).into(),
                mask.clone(),
                LocalState::empty(),
                parts.staged_ledger_hash.0.to_field(),
                (&parts.pending_coinbase).into(),
                |key| states.get(&key).cloned().unwrap(),
            )?,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mina_p2p_messages::v2::MinaBaseLedgerHash0StableV1;

    use super::*;

    #[test]
    fn test_ledger_hash() {
        IntoIterator::into_iter([
            (
                LedgerAddress::root(),
                "jwDnyiwZ4a3izRefHAKSrt5U5K6gS6p6G58kWdnJ5wYJqHUyaVd",
                "jxt96SwXGrpiyq9AVZ1B7vbt14NwcxXHgnBjArPpfM3dm216oy6",
                "jwq3nCDr8XejL8HKDxR5qVhFJbKoUTGZgtLBZCp3MrqLTnqmjdP",
            ),
            (
                LedgerAddress::root(),
                "jwLYDFqnEzBXjmwKBorWMiRnxYoisU69ZGEvS7g9Bev4WudZsxC",
                "jxoNu92jnreeAXnYxhMChEM22Bf5yAEw8pa7NRNitUnyKvzjcDs",
                "jwq3nCDr8XejL8HKDxR5qVhFJbKoUTGZgtLBZCp3MrqLTnqmjdP",
            ),
        ])
        .map(|(addr, expected_hash, left, right)| {
            let left: LedgerHash = left.parse().unwrap();
            let right: LedgerHash = right.parse().unwrap();
            (addr, expected_hash, left, right)
        })
        .for_each(|(address, expected_hash, left, right)| {
            let hash = ledger_hash(address.length(), left.0.to_field(), right.0.to_field());
            let hash: LedgerHash = MinaBaseLedgerHash0StableV1(hash.into()).into();
            assert_eq!(hash.to_string(), expected_hash);
        });
    }
}

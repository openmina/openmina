use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use ledger::{
    scan_state::{
        currency::{Amount, Fee, Slot},
        scan_state::ConstraintConstants,
        transaction_logic::{local_state::LocalState, protocol_state::protocol_state_view},
    },
    staged_ledger::{diff::Diff, staged_ledger::StagedLedger},
    verifier::Verifier,
    AccountIndex, BaseLedger, Mask, TreeVersion,
};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseStagedLedgerHashStableV1,
};
use mina_signer::CompressedPubKey;
use shared::block::ArcBlockWithHash;

use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerService;
use crate::transition_frontier::TransitionFrontierService;

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
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
    sync: LedgerSyncState,
}

#[derive(Default)]
struct LedgerSyncState {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
}

impl LedgerCtx {
    fn staged_ledger(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        match self.staged_ledgers.get_mut(&hash) {
            Some(v) => Some(v),
            None => self.sync.staged_ledger(&hash),
        }
    }
}

impl LedgerSyncState {
    fn snarked_ledger(&mut self, hash: LedgerHash) -> &mut Mask {
        self.snarked_ledgers.entry(hash.clone()).or_insert_with(|| {
            let mut ledger = Mask::create(LEDGER_DEPTH);
            ledger.set_cached_hash_unchecked(&LedgerAddress::root(), hash.0.to_field());
            ledger
        })
    }

    fn staged_ledger(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        self.staged_ledgers.get_mut(&hash)
    }
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
    fn hashes_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        (left, right): (LedgerHash, LedgerHash),
    ) -> Result<(), ()> {
        let (left, right) = (left.0.to_field(), right.0.to_field());
        let hash = ledger_hash(parent.length(), left, right);

        let mask = self.ctx_mut().sync.snarked_ledger(snarked_ledger_hash);

        if hash != mask.get_inner_hash_at_addr(parent.clone())? {
            return Err(());
        }

        mask.set_cached_hash_unchecked(&parent.child_left(), left);
        mask.set_cached_hash_unchecked(&parent.child_right(), right);

        Ok(())
    }

    fn accounts_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
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
        let mask = self.ctx_mut().sync.snarked_ledger(snarked_ledger_hash);

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
        snarked_ledger_hash: LedgerHash,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    ) -> Result<(), String> {
        let snarked_ledger = self.ctx_mut().sync.snarked_ledger(snarked_ledger_hash);
        // TODO(binier): TMP. Remove for prod version.
        snarked_ledger
            .validate_inner_hashes()
            .map_err(|_| "downloaded hash and recalculated mismatch".to_owned())?;

        let states = parts
            .needed_blocks
            .iter()
            .map(|state| (state.hash().to_fp().unwrap(), state.clone()))
            .collect::<BTreeMap<_, _>>();

        let mask = snarked_ledger.make_child();
        let staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            &CONSTRAINT_CONSTANTS,
            Verifier,
            (&parts.scan_state).into(),
            mask,
            LocalState::empty(),
            parts.staged_ledger_hash.0.to_field(),
            (&parts.pending_coinbase).into(),
            |key| states.get(&key).cloned().unwrap(),
        )?;

        self.ctx_mut()
            .sync
            .staged_ledgers
            .insert(parts.staged_ledger_hash.clone(), staged_ledger);

        Ok(())
    }
}

impl<T: LedgerService> TransitionFrontierService for T {
    fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String> {
        let mut staged_ledger = self
            .ctx_mut()
            .staged_ledger(&pred_block.staged_ledger_hash())
            .ok_or_else(|| "parent staged ledger missing")?
            .clone();

        let global_slot = block.global_slot();
        let prev_protocol_state = &pred_block.header().protocol_state;
        let prev_state_view = protocol_state_view(prev_protocol_state);

        let consensus_state = &block.header().protocol_state.body.consensus_state;
        let coinbase_receiver: CompressedPubKey = (&consensus_state.coinbase_receiver).into();
        let _supercharge_coinbase = consensus_state.supercharge_coinbase;

        // FIXME: Using `supercharge_coinbase` (from block) above does not work
        let supercharge_coinbase = false;

        let diff: Diff = (&block.block.body.staged_ledger_diff).into();

        let result = staged_ledger
            .apply(
                None,
                &CONSTRAINT_CONSTANTS,
                Slot::from_u32(global_slot),
                diff,
                (),
                &Verifier,
                &prev_state_view,
                ledger::scan_state::protocol_state::hashes(prev_protocol_state),
                coinbase_receiver,
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;
        let ledger_hashes = MinaBaseStagedLedgerHashStableV1::from(&result.hash_after_applying);

        // TODO(binier): return error if not matching.
        let expected_ledger_hashes = block.staged_ledger_hashes();
        if &ledger_hashes != expected_ledger_hashes {
            panic!("staged ledger hash mismatch. found: {ledger_hashes:?}, expected: {expected_ledger_hashes:?}");
        }

        let ledger_hash = block.staged_ledger_hash();
        self.ctx_mut()
            .sync
            .staged_ledgers
            .insert(ledger_hash.clone(), staged_ledger);

        Ok(())
    }

    fn commit(&mut self, ledgers_to_keep: BTreeSet<LedgerHash>) {
        let ctx = self.ctx_mut();

        ctx.snarked_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        ctx.snarked_ledgers.extend(
            std::mem::take(&mut ctx.sync.snarked_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );

        ctx.staged_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        ctx.staged_ledgers.extend(
            std::mem::take(&mut ctx.sync.staged_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );
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

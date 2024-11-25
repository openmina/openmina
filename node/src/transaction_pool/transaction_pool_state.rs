use ledger::{
    scan_state::{
        currency::{Amount, Nonce, Slot},
        transaction_logic::valid::UserCommand,
    },
    transaction_pool::{Config, ValidCommandWithHash},
    AccountId,
};
use mina_p2p_messages::v2::{self, TransactionHash};
use openmina_core::{consensus::ConsensusConstants, distributed_pool::DistributedPool};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use super::TransactionPoolAction;

pub(super) type PendingId = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPoolState {
    // TODO(binier): ideally this and `.pool` should be merged together.
    pub(super) dpool: DistributedPool<TransactionState, v2::TransactionHash>,
    pub(super) pool: ledger::transaction_pool::TransactionPool,
    pub(super) pending_actions: BTreeMap<PendingId, TransactionPoolAction>,
    pub(super) pending_id: PendingId,
    pub(super) best_tip_hash: Option<v2::LedgerHash>,
    /// For debug only
    #[serde(skip)]
    pub(super) file: Option<std::fs::File>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionState {
    pub time: redux::Timestamp,
    pub hash: TransactionHash,
}

impl AsRef<TransactionHash> for TransactionState {
    fn as_ref(&self) -> &TransactionHash {
        &self.hash
    }
}

impl Clone for TransactionPoolState {
    fn clone(&self) -> Self {
        Self {
            dpool: self.dpool.clone(),
            pool: self.pool.clone(),
            pending_actions: self.pending_actions.clone(),
            pending_id: self.pending_id,
            best_tip_hash: self.best_tip_hash.clone(),
            file: None,
        }
    }
}

impl TransactionPoolState {
    pub fn new(config: Config, consensus_constants: &ConsensusConstants) -> Self {
        Self {
            dpool: Default::default(),
            pool: ledger::transaction_pool::TransactionPool::new(config, consensus_constants),
            pending_actions: Default::default(),
            pending_id: 0,
            best_tip_hash: None,
            file: None,
        }
    }

    pub fn size(&self) -> usize {
        self.pool.size()
    }

    pub fn get(&self, hash: &TransactionHash) -> Option<&UserCommand> {
        self.pool.pool.get(hash).map(|v| &v.data)
    }

    pub fn transactions(&mut self, limit: usize) -> Vec<ValidCommandWithHash> {
        self.pool.transactions(limit)
    }

    pub fn list_includable_transactions(&self, limit: usize) -> Vec<ValidCommandWithHash> {
        self.pool.list_includable_transactions(limit)
    }

    pub fn get_all_transactions(&self) -> Vec<ValidCommandWithHash> {
        self.pool.get_all_transactions()
    }

    pub fn get_pending_amount_and_nonce(&self) -> HashMap<AccountId, (Option<Nonce>, Amount)> {
        self.pool.get_pending_amount_and_nonce()
    }

    fn next_pending_id(&mut self) -> PendingId {
        let id = self.pending_id;
        self.pending_id = self.pending_id.wrapping_add(1);
        id
    }

    pub(super) fn make_action_pending(&mut self, action: &TransactionPoolAction) -> PendingId {
        let id = self.next_pending_id();
        self.pending_actions.insert(id, action.clone());
        id
    }

    #[allow(dead_code)]
    fn save_actions(state: &mut crate::Substate<Self>) {
        let substate = state.get_substate_mut().unwrap();
        if substate.file.is_none() {
            let mut file = std::fs::File::create("/tmp/pool.bin").unwrap();
            postcard::to_io(&state.unsafe_get_state(), &mut file).unwrap();
            let substate = state.get_substate_mut().unwrap();
            substate.file = Some(file);
        }
    }

    pub(super) fn global_slots(state: &crate::State) -> Option<(Slot, Slot)> {
        Some((
            Slot::from_u32(state.cur_global_slot()?),
            Slot::from_u32(state.cur_global_slot_since_genesis()?),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::TransactionPoolActionWithMeta;
    use super::*;
    use crate::State;
    use redux::Dispatcher;

    #[allow(unused)]
    #[test]
    fn test_replay_pool() {
        let vec = std::fs::read("/tmp/pool.bin").unwrap();
        let slice = vec.as_slice();

        let (mut state, rest) = postcard::take_from_bytes::<State>(slice).unwrap();
        let mut slice = rest;

        while let Ok((action, rest)) =
            postcard::take_from_bytes::<TransactionPoolActionWithMeta>(slice)
        {
            slice = rest;

            let mut dispatcher = Dispatcher::new();
            let state = crate::Substate::<TransactionPoolState>::new(&mut state, &mut dispatcher);
            let (action, meta) = action.split();

            TransactionPoolState::handle_action(state, meta.with_action(&action));
        }
    }
}

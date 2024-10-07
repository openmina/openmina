use openmina_core::block::ArcBlockWithHash;
use openmina_core::consensus::ConsensusConstants;
use openmina_core::{constants::constraint_constants, error, ChainId};
use p2p::bootstrap::P2pNetworkKadBootstrapState;
use p2p::network::identify::P2pNetworkIdentifyState;
use p2p::{P2pConfig, P2pNetworkSchedulerState, P2pPeerState, P2pPeerStatusReady, PeerId};
use redux::{ActionMeta, EnablingCondition, Timestamp};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyState;
use snark::user_command_verify::SnarkUserCommandVerifyState;
use snark::work_verify::SnarkWorkVerifyState;

pub use crate::block_producer::BlockProducerState;
use crate::config::GlobalConfig;
pub use crate::consensus::ConsensusState;
use crate::external_snark_worker::ExternalSnarkWorkers;
pub use crate::ledger::LedgerState;
pub use crate::p2p::P2pState;
pub use crate::rpc::RpcState;
pub use crate::snark::SnarkState;
pub use crate::snark_pool::candidate::SnarkPoolCandidatesState;
pub use crate::snark_pool::SnarkPoolState;
use crate::transaction_pool::TransactionPoolState;
use crate::transition_frontier::genesis::TransitionFrontierGenesisState;
use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedState;
use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedState;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerState;
use crate::transition_frontier::sync::TransitionFrontierSyncState;
pub use crate::transition_frontier::TransitionFrontierState;
pub use crate::watched_accounts::WatchedAccountsState;
use crate::ActionWithMeta;
pub use crate::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub config: GlobalConfig,

    pub p2p: P2p,
    pub ledger: LedgerState,
    pub snark: SnarkState,
    pub consensus: ConsensusState,
    pub transition_frontier: TransitionFrontierState,
    pub snark_pool: SnarkPoolState,
    pub external_snark_worker: ExternalSnarkWorkers,
    pub transaction_pool: TransactionPoolState,
    pub block_producer: BlockProducerState,
    pub rpc: RpcState,

    pub watched_accounts: WatchedAccountsState,

    // TODO(binier): include action kind in `last_action`.
    last_action: ActionMeta,
    applied_actions_count: u64,
}

// Substate accessors that will be used in reducers
use openmina_core::impl_substate_access;

impl_substate_access!(State, SnarkState, snark);
impl_substate_access!(State, SnarkBlockVerifyState, snark.block_verify);
impl_substate_access!(State, SnarkWorkVerifyState, snark.work_verify);
impl_substate_access!(
    State,
    SnarkUserCommandVerifyState,
    snark.user_command_verify
);
impl_substate_access!(State, ConsensusState, consensus);
impl_substate_access!(State, TransitionFrontierState, transition_frontier);
impl_substate_access!(State, TransactionPoolState, transaction_pool);
impl_substate_access!(
    State,
    TransitionFrontierGenesisState,
    transition_frontier.genesis
);
impl_substate_access!(State, TransitionFrontierSyncState, transition_frontier.sync);
impl_substate_access!(State, SnarkPoolState, snark_pool);
impl_substate_access!(State, SnarkPoolCandidatesState, snark_pool.candidates);
impl_substate_access!(State, ExternalSnarkWorkers, external_snark_worker);
impl_substate_access!(State, BlockProducerState, block_producer);
impl_substate_access!(State, RpcState, rpc);
impl_substate_access!(State, WatchedAccountsState, watched_accounts);

impl openmina_core::SubstateAccess<P2pState> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&P2pState> {
        self.p2p
            .ready()
            .ok_or_else(|| "P2P state unavailable. P2P layer is not ready".to_owned())
    }

    fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut P2pState> {
        self.p2p
            .ready_mut()
            .ok_or_else(|| "P2P state unavailable. P2P layer is not ready".to_owned())
    }
}

impl openmina_core::SubstateAccess<TransitionFrontierSyncLedgerState> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&TransitionFrontierSyncLedgerState> {
        self.transition_frontier
            .sync
            .ledger()
            .ok_or_else(|| "Ledger sync state unavailable".to_owned())
    }

    fn substate_mut(
        &mut self,
    ) -> openmina_core::SubstateResult<&mut TransitionFrontierSyncLedgerState> {
        self.transition_frontier
            .sync
            .ledger_mut()
            .ok_or_else(|| "Ledger sync state unavailable".to_owned())
    }
}

impl openmina_core::SubstateAccess<TransitionFrontierSyncLedgerSnarkedState> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&TransitionFrontierSyncLedgerSnarkedState> {
        self.transition_frontier
            .sync
            .ledger()
            .ok_or_else(|| {
                "Snarked ledger state unavailable. Ledger sync state unavailable".to_owned()
            })?
            .snarked()
            .ok_or_else(|| "Snarked ledger state unavailable".to_owned())
    }

    fn substate_mut(
        &mut self,
    ) -> openmina_core::SubstateResult<&mut TransitionFrontierSyncLedgerSnarkedState> {
        self.transition_frontier
            .sync
            .ledger_mut()
            .ok_or_else(|| {
                "Snarked ledger state unavailable. Ledger sync state unavailable".to_owned()
            })?
            .snarked_mut()
            .ok_or_else(|| "Snarked ledger state unavailable".to_owned())
    }
}

impl openmina_core::SubstateAccess<TransitionFrontierSyncLedgerStagedState> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&TransitionFrontierSyncLedgerStagedState> {
        self.transition_frontier
            .sync
            .ledger()
            .ok_or_else(|| {
                "Staged ledger state unavailable. Ledger sync state unavailable".to_owned()
            })?
            .staged()
            .ok_or_else(|| "Staged ledger state unavailable".to_owned())
    }

    fn substate_mut(
        &mut self,
    ) -> openmina_core::SubstateResult<&mut TransitionFrontierSyncLedgerStagedState> {
        self.transition_frontier
            .sync
            .ledger_mut()
            .ok_or_else(|| {
                "Staged ledger state unavailable. Ledger sync state unavailable".to_owned()
            })?
            .staged_mut()
            .ok_or_else(|| "Staged ledger state unavailable".to_owned())
    }
}

macro_rules! impl_p2p_state_access {
    ($state:ty, $substate_type:ty) => {
        impl openmina_core::SubstateAccess<$substate_type> for $state {
            fn substate(&self) -> openmina_core::SubstateResult<&$substate_type> {
                let substate: &P2pState = self.substate()?;
                substate.substate()
            }

            fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut $substate_type> {
                let substate: &mut P2pState = self.substate_mut()?;
                substate.substate_mut()
            }
        }
    };
}

impl_p2p_state_access!(State, P2pNetworkIdentifyState);
impl_p2p_state_access!(State, p2p::P2pNetworkState);
impl_p2p_state_access!(State, P2pNetworkKadBootstrapState);
impl_p2p_state_access!(State, p2p::P2pNetworkKadState);
impl_p2p_state_access!(State, P2pNetworkSchedulerState);
impl_p2p_state_access!(State, p2p::P2pLimits);
impl_p2p_state_access!(State, p2p::P2pNetworkPubsubState);
impl_p2p_state_access!(State, p2p::P2pConfig);

impl p2p::P2pStateTrait for State {}

pub type Substate<'a, S> = openmina_core::Substate<'a, crate::Action, State, S>;

impl State {
    pub fn new(config: Config, constants: &ConsensusConstants, now: Timestamp) -> Self {
        Self {
            p2p: P2p::Pending(config.p2p),
            ledger: LedgerState::new(config.ledger),
            snark_pool: SnarkPoolState::new(),
            snark: SnarkState::new(config.snark),
            consensus: ConsensusState::new(),
            transition_frontier: TransitionFrontierState::new(config.transition_frontier),
            external_snark_worker: ExternalSnarkWorkers::new(now),
            block_producer: BlockProducerState::new(now, config.block_producer),
            rpc: RpcState::new(),
            transaction_pool: TransactionPoolState::new(config.tx_pool, constants),

            watched_accounts: WatchedAccountsState::new(),

            config: config.global,
            last_action: ActionMeta::zero_custom(now),
            applied_actions_count: 0,
        }
    }

    pub fn last_action(&self) -> &ActionMeta {
        &self.last_action
    }

    /// Latest time observed by the state machine.
    ///
    /// Only updated when action is dispatched and reducer is executed.
    #[inline(always)]
    pub fn time(&self) -> Timestamp {
        self.last_action.time()
    }

    /// Must be called in the global reducer as the last thing only once
    /// and only there!
    pub fn action_applied(&mut self, action: &ActionWithMeta) {
        self.last_action = action.meta().clone();
        self.applied_actions_count += 1;
    }

    pub fn genesis_block(&self) -> Option<ArcBlockWithHash> {
        self.transition_frontier
            .genesis
            .block_with_real_or_dummy_proof()
    }

    fn cur_slot(&self, initial_slot: impl FnOnce(&ArcBlockWithHash) -> u32) -> Option<u32> {
        let best_tip = self.transition_frontier.best_tip()?;
        let best_tip_ms = u64::from(best_tip.timestamp()) / 1_000_000;
        let now_ms = u64::from(self.time()) / 1_000_000;
        let ms = now_ms.saturating_sub(best_tip_ms);
        let slots = ms / constraint_constants().block_window_duration_ms;
        Some(initial_slot(best_tip) + slots as u32)
    }

    /// Current global slot based on constants and current time.
    ///
    /// It's not equal to global slot of the best tip.
    pub fn cur_global_slot(&self) -> Option<u32> {
        self.cur_slot(|b| b.global_slot())
    }

    pub fn cur_global_slot_since_genesis(&self) -> Option<u32> {
        self.cur_slot(|b| b.global_slot_since_genesis())
    }

    pub fn current_epoch(&self) -> Option<u32> {
        let slots_per_epoch = self.genesis_block()?.constants().slots_per_epoch.as_u32();
        Some(self.cur_global_slot()? / slots_per_epoch)
    }

    pub fn should_produce_blocks_after_genesis(&self) -> bool {
        self.block_producer.is_enabled()
            && self.genesis_block().map_or(false, |b| {
                let slot = &b.consensus_state().curr_global_slot_since_hard_fork;
                let epoch = slot.slot_number.as_u32() / slot.slots_per_epoch.as_u32();
                self.current_epoch() <= Some(epoch)
            })
    }

    pub fn should_log_node_id(&self) -> bool {
        self.config.testing_run
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2p {
    Pending(P2pConfig),
    Ready(P2pState),
}

#[derive(Debug, thiserror::Error)]
pub enum P2pInitializationError {
    #[error("p2p is already initialized")]
    AlreadyInitialized,
}

#[macro_export]
macro_rules! p2p_ready {
    ($p2p:expr, $time:expr) => {
        p2p_ready!($p2p, "", $time)
    };
    ($p2p:expr, $reason:expr, $time:expr) => {
        match $p2p.ready() {
            Some(v) => v,
            None => {
                //panic!("p2p is not ready: {:?}\nline: {}", $reason, line!());
                openmina_core::error!($time; "p2p is not initialized: {}", $reason);
                return;
            }
        }
    };
}

impl P2p {
    pub fn config(&self) -> &P2pConfig {
        match self {
            P2p::Pending(config) => config,
            P2p::Ready(p2p_state) => &p2p_state.config,
        }
    }

    // TODO: add chain id
    pub fn initialize(&mut self, chain_id: &ChainId) -> Result<(), P2pInitializationError> {
        let P2p::Pending(config) = self else {
            return Err(P2pInitializationError::AlreadyInitialized);
        };
        *self = P2p::Ready(P2pState::new(config.clone(), chain_id));
        Ok(())
    }

    pub fn ready(&self) -> Option<&P2pState> {
        if let P2p::Ready(state) = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn ready_mut(&mut self) -> Option<&mut P2pState> {
        if let P2p::Ready(state) = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn unwrap(&self) -> &P2pState {
        self.ready().expect("p2p is not initialized")
    }

    pub fn is_enabled<T>(&self, action: &T, time: Timestamp) -> bool
    where
        T: EnablingCondition<P2pState>,
    {
        match self {
            P2p::Pending(_) => false,
            P2p::Ready(p2p_state) => action.is_enabled(p2p_state, time),
        }
    }

    pub fn my_id(&self) -> PeerId {
        match self {
            P2p::Pending(config) => &config.identity_pub_key,
            P2p::Ready(state) => &state.config.identity_pub_key,
        }
        .peer_id()
    }

    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerState> {
        self.ready().and_then(|p2p| p2p.peers.get(peer_id))
    }

    pub fn get_ready_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerStatusReady> {
        self.ready().and_then(|p2p| p2p.get_ready_peer(peer_id))
    }

    pub fn ready_peers(&self) -> Vec<PeerId> {
        self.ready_peers_iter()
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }

    pub fn ready_peers_iter(&self) -> ReadyPeersIter {
        ReadyPeersIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct ReadyPeersIter<'a>(Option<std::collections::btree_map::Iter<'a, PeerId, P2pPeerState>>);

impl<'a> ReadyPeersIter<'a> {
    fn new(p2p: &'a P2p) -> Self {
        ReadyPeersIter(p2p.ready().map(|p2p| p2p.peers.iter()))
    }
}

impl<'a> Iterator for ReadyPeersIter<'a> {
    type Item = (&'a PeerId, &'a P2pPeerStatusReady);

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.0.as_mut()?;
        Some(loop {
            let (peer_id, state) = iter.next()?;
            if let Some(ready) = state.status.as_ready() {
                break (peer_id, ready);
            }
        })
    }
}

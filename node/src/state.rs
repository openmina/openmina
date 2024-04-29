use openmina_core::{constants::CONSTRAINT_CONSTANTS, error, ChainId};
use p2p::{P2pConfig, P2pPeerState, P2pPeerStatusReady, PeerId};
use redux::{ActionMeta, EnablingCondition, Timestamp};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyState;
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
use crate::transition_frontier::genesis::TransitionFrontierGenesisState;
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
    pub block_producer: BlockProducerState,
    pub rpc: RpcState,

    pub watched_accounts: WatchedAccountsState,

    // TODO(binier): include action kind in `last_action`.
    last_action: ActionMeta,
    applied_actions_count: u64,
}

// Substate accessors that will be used in reducers
use openmina_core::impl_substate_access;

impl_substate_access!(State, P2pState, p2p);
impl_substate_access!(State, p2p::P2pNetworkState, p2p.network);
impl_substate_access!(State, SnarkState, snark);
impl_substate_access!(State, SnarkBlockVerifyState, snark.block_verify);
impl_substate_access!(State, SnarkWorkVerifyState, snark.work_verify);
impl_substate_access!(State, ConsensusState, consensus);
impl_substate_access!(State, TransitionFrontierState, transition_frontier);
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

pub type Substate<'a, S> = openmina_core::Substate<'a, crate::Action, State, S>;

impl State {
    pub fn new(config: Config) -> Self {
        let now = Timestamp::global_now();
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

            watched_accounts: WatchedAccountsState::new(),

            config: config.global,
            last_action: ActionMeta::zero_custom(now),
            applied_actions_count: 0,
        }
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

    /// Current global slot based on constants and current time.
    ///
    /// It's not equal to global slot of the best tip.
    pub fn cur_global_slot(&self) -> Option<u32> {
        let best_tip = self.transition_frontier.best_tip()?;
        let best_tip_ms = u64::from(best_tip.timestamp()) / 1_000_000;
        let now_ms = u64::from(self.time()) / 1_000_000;
        let ms = now_ms.saturating_sub(best_tip_ms);
        let slots = ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;

        Some(best_tip.global_slot() + (slots as u32))
    }

    pub fn current_epoch(&self) -> Option<u32> {
        // TODO: Should not be hardcoded
        const SLOTS_PER_EPOCH: u32 = 7140;
        let current_global_slot = self.cur_global_slot()?;
        Some(current_global_slot / SLOTS_PER_EPOCH)
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
        let Some(iter) = self.0.as_mut() else {
            return None;
        };
        Some(loop {
            let (peer_id, state) = iter.next()?;
            if let Some(ready) = state.status.as_ready() {
                break (peer_id, ready);
            }
        })
    }
}

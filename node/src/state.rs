use std::sync::Arc;
use std::time::Duration;

use mina_p2p_messages::v2::{MinaBaseUserCommandStableV2, MinaBlockBlockStableV2};
use openmina_core::constants::PROTOCOL_VERSION;
use rand::prelude::*;

use openmina_core::block::BlockWithHash;
use openmina_core::requests::RpcId;
use openmina_core::snark::{Snark, SnarkInfo};
use openmina_core::{
    block::ArcBlockWithHash, consensus::ConsensusConstants, constants::constraint_constants, error,
    snark::SnarkJobCommitment, ChainId,
};
use p2p::channels::rpc::{P2pRpcId, P2pRpcRequest, P2pRpcResponse};
use p2p::channels::streaming_rpc::P2pStreamingRpcResponseFull;
use p2p::connection::outgoing::P2pConnectionOutgoingError;
use p2p::connection::P2pConnectionResponse;
use p2p::{
    bootstrap::P2pNetworkKadBootstrapState, network::identify::P2pNetworkIdentifyState,
    P2pCallbacks, P2pConfig, P2pNetworkSchedulerState, P2pPeerState, P2pPeerStatusReady, PeerId,
};
use redux::{ActionMeta, EnablingCondition, Timestamp};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyState;
use snark::user_command_verify::SnarkUserCommandVerifyState;
use snark::work_verify::SnarkWorkVerifyState;

use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorState;
pub use crate::block_producer::BlockProducerState;
pub use crate::consensus::ConsensusState;
use crate::external_snark_worker::{ExternalSnarkWorker, ExternalSnarkWorkers};
use crate::ledger::read::LedgerReadState;
use crate::ledger::write::LedgerWriteState;
pub use crate::ledger::LedgerState;
use crate::p2p::callbacks::P2pCallbacksAction;
pub use crate::p2p::P2pState;
pub use crate::rpc::RpcState;
pub use crate::snark::SnarkState;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
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
pub use crate::Config;
use crate::{config::GlobalConfig, SnarkPoolAction};
use crate::{ActionWithMeta, ConsensusAction, RpcAction, TransactionPoolAction};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockPrevalidationError {
    GenesisNotReady,
    ReceivedTooEarly {
        current_global_slot: u32,
        block_global_slot: u32,
    },
    ReceivedTooLate {
        current_global_slot: u32,
        block_global_slot: u32,
        delta: u32,
    },
    InvalidGenesisProtocolState,
    InvalidProtocolVersion,
    MismatchedProtocolVersion,
    ConsantsMismatch,
    InvalidDeltaBlockChainProof,
}

// Substate accessors that will be used in reducers
use openmina_core::{bug_condition, impl_substate_access, SubstateAccess};

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
impl_substate_access!(State, ExternalSnarkWorker, external_snark_worker.0);
impl_substate_access!(State, LedgerState, ledger);
impl_substate_access!(State, LedgerReadState, ledger.read);
impl_substate_access!(State, LedgerWriteState, ledger.write);

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

impl SubstateAccess<BlockProducerVrfEvaluatorState> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&BlockProducerVrfEvaluatorState> {
        self.block_producer
            .as_ref()
            .map(|state| &state.vrf_evaluator)
            .ok_or_else(|| "Block producer VRF evaluator state unavailable".to_owned())
    }

    fn substate_mut(
        &mut self,
    ) -> openmina_core::SubstateResult<&mut BlockProducerVrfEvaluatorState> {
        self.block_producer
            .as_mut()
            .map(|state| &mut state.vrf_evaluator)
            .ok_or_else(|| "Block producer VRF evaluator state unavailable".to_owned())
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

impl SubstateAccess<State> for State {
    fn substate(&self) -> openmina_core::SubstateResult<&State> {
        Ok(self)
    }

    fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut State> {
        Ok(self)
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

    pub fn pseudo_rng(&self) -> StdRng {
        StdRng::seed_from_u64(self.time().into())
    }

    /// Must be called in the global reducer as the last thing only once
    /// and only there!
    pub fn action_applied(&mut self, action: &ActionWithMeta) {
        self.last_action = action.meta().clone();
        self.applied_actions_count = self.applied_actions_count.checked_add(1).expect("overflow");
    }

    pub fn genesis_block(&self) -> Option<ArcBlockWithHash> {
        self.transition_frontier
            .genesis
            .block_with_real_or_dummy_proof()
    }

    fn cur_slot(&self, initial_slot: impl FnOnce(&ArcBlockWithHash) -> u32) -> Option<u32> {
        let genesis = self.genesis_block()?;
        let initial_ms = u64::from(genesis.timestamp()) / 1_000_000;
        let now_ms = u64::from(self.time()) / 1_000_000;
        let ms = now_ms.saturating_sub(initial_ms);
        let slots = ms
            .checked_div(constraint_constants().block_window_duration_ms)
            .expect("division by 0");
        Some(
            initial_slot(&genesis)
                .checked_add(slots as u32)
                .expect("overflow"),
        )
    }

    /// Current global slot based on constants and current time.
    ///
    /// It's not equal to global slot of the best tip.
    pub fn cur_global_slot(&self) -> Option<u32> {
        self.cur_slot(|b| b.global_slot())
    }

    pub fn current_slot(&self) -> Option<u32> {
        let slots_per_epoch = self.genesis_block()?.constants().slots_per_epoch.as_u32();
        Some(
            self.cur_global_slot()?
                .checked_rem(slots_per_epoch)
                .expect("division by 0"),
        )
    }

    pub fn cur_global_slot_since_genesis(&self) -> Option<u32> {
        self.cur_slot(|b| b.global_slot_since_genesis())
    }

    pub fn current_epoch(&self) -> Option<u32> {
        let slots_per_epoch = self.genesis_block()?.constants().slots_per_epoch.as_u32();
        Some(
            self.cur_global_slot()?
                .checked_div(slots_per_epoch)
                .expect("division by 0"),
        )
    }

    pub fn producing_block_after_genesis(&self) -> bool {
        #[allow(clippy::arithmetic_side_effects)]
        let two_mins_in_future = self.time() + Duration::from_secs(2 * 60);
        self.block_producer.with(false, |bp| {
            bp.current.won_slot_should_produce(two_mins_in_future)
        }) && self.genesis_block().map_or(false, |b| {
            let slot = &b.consensus_state().curr_global_slot_since_hard_fork;
            let epoch = slot
                .slot_number
                .as_u32()
                .checked_div(slot.slots_per_epoch.as_u32())
                .expect("division by 0");
            self.current_epoch() <= Some(epoch)
        })
    }

    pub fn prevalidate_block(
        &self,
        block: &ArcBlockWithHash,
        allow_block_too_late: bool,
    ) -> Result<(), BlockPrevalidationError> {
        let Some((genesis, cur_global_slot)) =
            None.or_else(|| Some((self.genesis_block()?, self.cur_global_slot()?)))
        else {
            // we don't have genesis block. This should be impossible
            // because we don't even know chain_id before we have genesis
            // block, so we can't be connected to any peers from which
            // we would receive a block.
            bug_condition!("Tried to prevalidate a block before the genesis block was ready");
            return Err(BlockPrevalidationError::GenesisNotReady);
        };

        // received_at_valid_time
        // https://github.com/minaprotocol/mina/blob/6af211ad58e9356f00ea4a636cea70aa8267c072/src/lib/consensus/proof_of_stake.ml#L2746
        {
            let block_global_slot = block.global_slot();

            let delta = genesis.constants().delta.as_u32();
            if cur_global_slot < block_global_slot {
                // Too_early
                return Err(BlockPrevalidationError::ReceivedTooEarly {
                    current_global_slot: cur_global_slot,
                    block_global_slot,
                });
            } else if !allow_block_too_late
                && cur_global_slot.saturating_sub(block_global_slot) > delta
            {
                // Too_late
                return Err(BlockPrevalidationError::ReceivedTooLate {
                    current_global_slot: cur_global_slot,
                    block_global_slot,
                    delta,
                });
            }
        }

        if block.header().genesis_state_hash() != genesis.hash() {
            return Err(BlockPrevalidationError::InvalidGenesisProtocolState);
        }

        let (protocol_versions_are_valid, protocol_version_matches_daemon) = {
            let min_transaction_version = 1.into();
            let v = &block.header().current_protocol_version;
            let nv = block
                .header()
                .proposed_protocol_version_opt
                .as_ref()
                .unwrap_or(v);

            // Our version values are unsigned, so there is no need to check that the
            // other parts are not negative.
            let valid = v.transaction >= min_transaction_version
                && nv.transaction >= min_transaction_version;
            let compatible = v.transaction == PROTOCOL_VERSION.transaction
                && v.network == PROTOCOL_VERSION.network;

            (valid, compatible)
        };

        if !protocol_versions_are_valid {
            return Err(BlockPrevalidationError::InvalidProtocolVersion);
        } else if !protocol_version_matches_daemon {
            return Err(BlockPrevalidationError::MismatchedProtocolVersion);
        }

        // NOTE: currently these cannot change between blocks, but that
        // may not always be true?
        if block.constants() != genesis.constants() {
            return Err(BlockPrevalidationError::ConsantsMismatch);
        }

        // TODO(tizoc): check for InvalidDeltaBlockChainProof
        // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/mina_block/validation.ml#L369
        // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/transition_chain_verifier/transition_chain_verifier.ml

        Ok(())
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

        let callbacks = Self::p2p_callbacks();
        *self = P2p::Ready(P2pState::new(config.clone(), callbacks, chain_id));
        Ok(())
    }

    fn p2p_callbacks() -> P2pCallbacks {
        P2pCallbacks {
            on_p2p_channels_transaction_libp2p_received: Some(redux::callback!(
                on_p2p_channels_transaction_libp2p_received(transaction: Box<MinaBaseUserCommandStableV2>) -> crate::Action{
                    TransactionPoolAction::StartVerify {
                        commands: std::iter::once(*transaction).collect(),
                        from_rpc: None
                    }
                }
            )),
            on_p2p_channels_snark_job_commitment_received: Some(redux::callback!(
                on_p2p_channels_snark_job_commitment_received((peer_id: PeerId, commitment: Box<SnarkJobCommitment>)) -> crate::Action{
                    SnarkPoolAction::CommitmentAdd { commitment: *commitment, sender: peer_id }
                }
            )),
            on_p2p_channels_snark_received: Some(redux::callback!(
                on_p2p_channels_snark_received((peer_id: PeerId, snark: Box<SnarkInfo>)) -> crate::Action{
                    SnarkPoolCandidateAction::InfoReceived { peer_id, info: *snark }
                }
            )),
            on_p2p_channels_snark_libp2p_received: Some(redux::callback!(
                on_p2p_channels_snark_libp2p_received((peer_id: PeerId, snark: Box<Snark>)) -> crate::Action{
                    SnarkPoolCandidateAction::WorkReceived { peer_id, work: *snark }
                }
            )),
            on_p2p_channels_streaming_rpc_ready: Some(redux::callback!(
                on_p2p_channels_streaming_rpc_ready(_var: ()) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsStreamingRpcReady
                }
            )),
            on_p2p_channels_best_tip_request_received: Some(redux::callback!(
                on_p2p_channels_best_tip_request_received(peer_id: PeerId) -> crate::Action{
                    P2pCallbacksAction::RpcRespondBestTip { peer_id }
                }
            )),
            on_p2p_disconnection_finish: Some(redux::callback!(
                on_p2p_disconnection_finish(peer_id: PeerId) -> crate::Action{
                    P2pCallbacksAction::P2pDisconnection { peer_id }
                }
            )),
            on_p2p_connection_outgoing_error: Some(redux::callback!(
                on_p2p_connection_outgoing_error((rpc_id: RpcId, error: P2pConnectionOutgoingError)) -> crate::Action{
                    RpcAction::P2pConnectionOutgoingError { rpc_id, error }
                }
            )),
            on_p2p_connection_outgoing_success: Some(redux::callback!(
                on_p2p_connection_outgoing_success(rpc_id: RpcId) -> crate::Action{
                    RpcAction::P2pConnectionOutgoingSuccess { rpc_id }
                }
            )),
            on_p2p_connection_incoming_error: Some(redux::callback!(
                on_p2p_connection_incoming_error((rpc_id: RpcId, error: String)) -> crate::Action{
                    RpcAction::P2pConnectionIncomingError { rpc_id, error }
                }
            )),
            on_p2p_connection_incoming_success: Some(redux::callback!(
                on_p2p_connection_incoming_success(rpc_id: RpcId) -> crate::Action{
                    RpcAction::P2pConnectionIncomingSuccess { rpc_id }
                }
            )),
            on_p2p_connection_incoming_answer_ready: Some(redux::callback!(
                on_p2p_connection_incoming_answer_ready((rpc_id: RpcId, peer_id: PeerId, answer: P2pConnectionResponse)) -> crate::Action{
                    RpcAction::P2pConnectionIncomingAnswerReady { rpc_id, answer, peer_id }
                }
            )),
            on_p2p_peer_best_tip_update: Some(redux::callback!(
                on_p2p_peer_best_tip_update(best_tip: BlockWithHash<Arc<MinaBlockBlockStableV2>>) -> crate::Action{
                    ConsensusAction::P2pBestTipUpdate { best_tip }
                }
            )),
            on_p2p_channels_rpc_ready: Some(redux::callback!(
                on_p2p_channels_rpc_ready(peer_id: PeerId) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsRpcReady { peer_id }
                }
            )),
            on_p2p_channels_rpc_timeout: Some(redux::callback!(
                on_p2p_channels_rpc_timeout((peer_id: PeerId, id: P2pRpcId)) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsRpcTimeout { peer_id, id }
                }
            )),
            on_p2p_channels_rpc_response_received: Some(redux::callback!(
                on_p2p_channels_rpc_response_received((peer_id: PeerId, id: P2pRpcId, response: Option<Box<P2pRpcResponse>>)) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsRpcResponseReceived { peer_id, id, response }
                }
            )),
            on_p2p_channels_rpc_request_received: Some(redux::callback!(
                on_p2p_channels_rpc_request_received((peer_id: PeerId, id: P2pRpcId, request: Box<P2pRpcRequest>)) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsRpcRequestReceived { peer_id, id, request }
                }
            )),
            on_p2p_channels_streaming_rpc_response_received: Some(redux::callback!(
                on_p2p_channels_streaming_rpc_response_received((peer_id: PeerId, id: P2pRpcId, response: Option<P2pStreamingRpcResponseFull>)) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsStreamingRpcResponseReceived { peer_id, id, response }
                }
            )),
            on_p2p_channels_streaming_rpc_timeout: Some(redux::callback!(
                on_p2p_channels_streaming_rpc_timeout((peer_id: PeerId, id: P2pRpcId)) -> crate::Action{
                    P2pCallbacksAction::P2pChannelsStreamingRpcTimeout { peer_id, id }
                }
            )),
        }
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

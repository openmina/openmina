use redux::{ActionMeta, Timestamp};
use serde::{Deserialize, Serialize};
use vrf::keypair_from_bs58_string;

pub use crate::block_producer::BlockProducerState;
use crate::config::GlobalConfig;
pub use crate::consensus::ConsensusState;
use crate::external_snark_worker::ExternalSnarkWorkers;
pub use crate::p2p::P2pState;
pub use crate::rpc::RpcState;
pub use crate::snark::SnarkState;
pub use crate::snark_pool::SnarkPoolState;
pub use crate::transition_frontier::TransitionFrontierState;
pub use crate::watched_accounts::WatchedAccountsState;
use crate::ActionWithMeta;
pub use crate::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub config: GlobalConfig,

    pub p2p: P2pState,
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

impl State {
    pub fn new(config: Config) -> Self {
        let now = Timestamp::global_now();
        let producer_address = config.global.producer.as_ref().map(|s| keypair_from_bs58_string(s).public.into_address());
        Self {
            p2p: P2pState::new(config.p2p),
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
}

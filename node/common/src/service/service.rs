use std::sync::Arc;

use node::{
    core::invariants::InvariantsState, event_source::Event, ledger::LedgerManager, stats::Stats,
    transition_frontier::genesis::GenesisConfig,
};
use rand::rngs::StdRng;
use sha3::{digest::core_api::XofReaderCoreWrapper, Shake256ReaderCore};

use crate::rpc::RpcReceiver;

use super::{
    block_producer::BlockProducerService,
    p2p::webrtc_with_libp2p::P2pServiceCtx,
    replay::ReplayerState,
    rpc::{RpcSender, RpcService},
    EventReceiver, EventSender,
};

pub struct NodeServiceCommon {
    pub rng_seed: [u8; 32],
    pub rng_ephemeral: XofReaderCoreWrapper<Shake256ReaderCore>,
    pub rng_static: XofReaderCoreWrapper<Shake256ReaderCore>,
    pub rng: StdRng,
    /// Events sent on this channel are retrieved and processed in the
    /// `event_source` state machine defined in the `openmina-node` crate.
    pub event_sender: EventSender,
    pub event_receiver: EventReceiver,

    pub ledger_manager: LedgerManager,
    pub block_producer: Option<BlockProducerService>,
    pub p2p: P2pServiceCtx,

    pub stats: Option<Stats>,
    pub rpc: RpcService,
    pub replayer: Option<ReplayerState>,
    pub invariants_state: InvariantsState,
}

impl NodeServiceCommon {
    pub fn event_sender(&self) -> &EventSender {
        &self.event_sender
    }

    pub fn event_receiver_with_rpc_receiver(&mut self) -> (&mut EventReceiver, &mut RpcReceiver) {
        (&mut self.event_receiver, self.rpc.req_receiver())
    }

    pub fn rpc_sender(&self) -> RpcSender {
        self.rpc.req_sender()
    }

    pub fn stats(&mut self) -> Option<&mut Stats> {
        self.stats.as_mut()
    }
}

impl redux::Service for NodeServiceCommon {}

impl redux::TimeService for NodeServiceCommon {
    fn monotonic_time(&mut self) -> redux::Instant {
        self.replayer
            .as_ref()
            .map(|v| v.next_monotonic_time())
            .unwrap_or_else(redux::Instant::now)
    }
}

impl node::service::EventSourceService for NodeServiceCommon {
    fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.try_next()
    }
}

impl node::service::LedgerService for NodeServiceCommon {
    fn ledger_manager(&self) -> &LedgerManager {
        &self.ledger_manager
    }

    fn force_sync_calls(&self) -> bool {
        self.replayer.is_some()
    }
}

impl node::service::TransitionFrontierGenesisService for NodeServiceCommon {
    fn load_genesis(&mut self, config: Arc<GenesisConfig>) {
        let res = match config.load() {
            Err(err) => Err(err.to_string()),
            Ok((mask, data)) => {
                self.ledger_manager.insert_genesis_ledger(mask);
                Ok(data)
            }
        };
        let _ = self.event_sender.send(Event::GenesisLoad(res));
    }
}

impl node::core::invariants::InvariantService for NodeServiceCommon {
    fn invariants_state(&mut self) -> &mut node::core::invariants::InvariantsState {
        &mut self.invariants_state
    }
}

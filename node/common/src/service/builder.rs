use ledger::proofs::gates::BlockProver;
use node::{
    account::AccountSecretKey,
    core::channels::mpsc,
    ledger::{LedgerCtx, LedgerManager},
    p2p::{
        identity::SecretKey as P2pSecretKey,
        service_impl::{
            webrtc_with_libp2p::{P2pServiceCtx, P2pServiceWebrtcWithLibp2p},
            TaskSpawner,
        },
    },
    stats::Stats,
};
use rand::{rngs::StdRng, SeedableRng};
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use crate::{
    rpc::{RpcSender, RpcService},
    EventReceiver, EventSender, NodeService,
};

use super::block_producer::BlockProducerService;

pub struct NodeServiceCommonBuilder {
    rng_seed: [u8; 32],
    rng: StdRng,
    /// Events sent on this channel are retrieved and processed in the
    /// `event_source` state machine defined in the `openmina-node` crate.
    event_sender: EventSender,
    event_receiver: EventReceiver,
    ledger_manager: Option<LedgerManager>,
    block_producer: Option<BlockProducerService>,
    p2p: Option<P2pServiceCtx>,
    gather_stats: bool,
    rpc: RpcService,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum NodeServiceCommonBuildError {
    #[error("ledger was never initialized! Please call: NodeServiceBuilder::ledger_init")]
    LedgerNotInit,
    #[error("p2p was never initialized! Please call: NodeServiceBuilder::p2p_init")]
    P2pNotInit,
}

impl NodeServiceCommonBuilder {
    pub fn new(rng_seed: [u8; 32]) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        Self {
            rng_seed,
            rng: StdRng::from_seed(rng_seed),
            event_sender,
            event_receiver: event_receiver.into(),
            ledger_manager: None,
            block_producer: None,
            p2p: None,
            rpc: RpcService::new(),
            gather_stats: false,
        }
    }

    pub fn event_sender(&self) -> &EventSender {
        &self.event_sender
    }

    pub fn rpc_sender(&self) -> RpcSender {
        self.rpc.req_sender()
    }

    pub fn ledger_init(&mut self) -> &mut Self {
        let mut ctx = LedgerCtx::default();
        ctx.set_event_sender(self.event_sender.clone());
        self.ledger_manager = Some(LedgerManager::spawn(ctx));
        self
    }

    pub fn block_producer_init(
        &mut self,
        provers: BlockProver,
        keypair: AccountSecretKey,
    ) -> &mut Self {
        self.block_producer = Some(BlockProducerService::start(
            provers,
            self.event_sender.clone(),
            keypair,
        ));
        self
    }

    pub fn p2p_init<S: TaskSpawner>(
        &mut self,
        secret_key: P2pSecretKey,
        task_spawner: S,
    ) -> &mut Self {
        self.p2p = Some(<NodeService as P2pServiceWebrtcWithLibp2p>::init(
            secret_key.clone(),
            task_spawner,
        ));
        self
    }

    pub fn gather_stats(&mut self) -> &mut Self {
        self.gather_stats = true;
        self
    }

    pub fn build(self) -> Result<NodeService, NodeServiceCommonBuildError> {
        let ledger_manager = self
            .ledger_manager
            .ok_or(NodeServiceCommonBuildError::LedgerNotInit)?;
        let p2p = self.p2p.ok_or(NodeServiceCommonBuildError::P2pNotInit)?;

        Ok(NodeService {
            rng_seed: self.rng_seed,
            rng_ephemeral: Shake256::default()
                .chain(self.rng_seed)
                .chain(b"ephemeral")
                .finalize_xof(),
            rng_static: Shake256::default()
                .chain(self.rng_seed)
                .chain(b"static")
                .finalize_xof(),
            rng: self.rng,
            event_sender: self.event_sender,
            event_receiver: self.event_receiver,
            ledger_manager,
            block_producer: self.block_producer,
            p2p,
            stats: self.gather_stats.then(Stats::new),
            rpc: self.rpc,
            recorder: Default::default(),
            replayer: None,
            invariants_state: Default::default(),
        })
    }
}

use ledger::proofs::provers::BlockProver;
use node::{
    account::AccountSecretKey, core::thread, p2p::identity::SecretKey as P2pSecretKey,
    service::Recorder,
};
pub use openmina_node_common::NodeServiceCommonBuildError;
use openmina_node_common::{
    p2p::TaskSpawner, rpc::RpcSender, EventSender, NodeServiceCommonBuilder,
};

use crate::{http_server, NodeService, P2pTaskSpawner};

pub struct NodeServiceBuilder {
    common: NodeServiceCommonBuilder,
    pub(super) recorder: Recorder,
    http_server_port: Option<u16>,
}

#[derive(thiserror::Error, derive_more::From, Debug, Clone)]
pub enum NodeServiceBuildError {
    #[error("error when building common parts of the service: {0}")]
    Common(NodeServiceCommonBuildError),
}

impl NodeServiceBuilder {
    pub fn new(rng_seed: [u8; 32]) -> Self {
        Self {
            common: NodeServiceCommonBuilder::new(rng_seed),
            recorder: Default::default(),
            http_server_port: None,
        }
    }

    pub fn event_sender(&self) -> &EventSender {
        self.common.event_sender()
    }

    pub fn rpc_sender(&self) -> RpcSender {
        self.common.rpc_sender()
    }

    pub fn ledger_init(&mut self) -> &mut Self {
        self.common.ledger_init();
        self
    }

    pub fn block_producer_init(
        &mut self,
        provers: BlockProver,
        keypair: AccountSecretKey,
    ) -> &mut Self {
        self.common.block_producer_init(provers, keypair);
        self
    }

    pub fn p2p_init(&mut self, secret_key: P2pSecretKey) -> &mut Self {
        self.common.p2p_init(secret_key, P2pTaskSpawner {});
        self
    }

    pub fn p2p_init_with_custom_task_spawner(
        &mut self,
        secret_key: P2pSecretKey,
        task_spawner: impl TaskSpawner,
    ) -> &mut Self {
        self.common.p2p_init(secret_key, task_spawner);
        self
    }

    pub fn gather_stats(&mut self) -> &mut Self {
        self.common.gather_stats();
        self
    }

    pub fn record(&mut self, recorder: Recorder) -> &mut Self {
        self.recorder = recorder;
        self
    }

    pub fn http_server_init(&mut self, port: u16) -> &mut Self {
        if let Some(cur_port) = self.http_server_port {
            panic!("trying to start http server on port `{port}`, when it's already running on port `{cur_port}`");
        }
        self.http_server_port = Some(port);
        let rpc_sender = self.rpc_sender();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        thread::Builder::new()
            .name("openmina_http_server".to_owned())
            .spawn(move || runtime.block_on(http_server::run(port, rpc_sender)))
            .unwrap();
        self
    }

    pub fn build(self) -> Result<NodeService, NodeServiceBuildError> {
        let mut service = self.common.build()?;
        service.recorder = self.recorder;
        Ok(service)
    }
}

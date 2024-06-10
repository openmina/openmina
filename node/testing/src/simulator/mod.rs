mod config;
pub use config::*;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

use std::{collections::BTreeSet, time::Duration};

use node::{ActionKind, BlockProducerConfig, SnarkerConfig, SnarkerStrategy, State};

use crate::{
    cluster::ClusterNodeId,
    node::{Node, RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::ListenerNode,
    scenarios::{ClusterRunner, RunCfg},
};

pub struct Simulator {
    initial_time: redux::Timestamp,
    config: SimulatorConfig,
}

impl Simulator {
    pub fn new(initial_time: redux::Timestamp, config: SimulatorConfig) -> Self {
        Self {
            initial_time,
            config,
        }
    }

    fn initial_time(&self) -> redux::Timestamp {
        self.initial_time
    }

    async fn seed_config_async(&self, _runner: &ClusterRunner<'_>) -> RustNodeTestingConfig {
        RustNodeTestingConfig {
            initial_time: self.initial_time(),
            genesis: self.config.genesis.clone(),
            max_peers: 1000,
            ask_initial_peers_interval: Duration::from_secs(60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: None,
            snark_worker: None,
            timeouts: Default::default(),
            libp2p_port: None,
            recorder: self.config.recorder.clone(),
        }
    }

    async fn wait_for_all_nodes_synced(&mut self, runner: &mut ClusterRunner<'_>) {
        eprintln!("waiting for all rust nodes to sync up");
        let is_synced = |state: &State| {
            state.transition_frontier.sync.is_synced()
                && state.transition_frontier.best_tip().is_some()
        };
        while !runner.nodes_iter().all(|(_, node)| is_synced(node.state())) {
            runner
                .run(RunCfg::default().action_handler(move |_, _, _, action| {
                    matches!(
                        action.action().kind(),
                        ActionKind::TransitionFrontierGenesisInject
                            | ActionKind::TransitionFrontierSynced
                    )
                }))
                .await
                .expect("error while waiting to sync genesis block from ocaml");
        }
        eprintln!("all rust nodes synced up");
    }

    async fn set_up_seed_nodes(&mut self, runner: &mut ClusterRunner<'_>) {
        eprintln!("setting up rust seed nodes: {}", self.config.seed_nodes);
        let seed_config = self.seed_config_async(runner).await;

        for _ in 0..(self.config.seed_nodes) {
            runner.add_rust_node(seed_config.clone());
        }

        self.wait_for_all_nodes_synced(runner).await;
    }

    fn seed_nodes_iter<'a>(
        &self,
        runner: &'a ClusterRunner<'_>,
    ) -> impl 'a + Iterator<Item = (ClusterNodeId, &'a Node)> {
        runner.nodes_iter().take(self.config.seed_nodes)
    }

    fn seed_node_dial_addrs(&self, runner: &ClusterRunner<'_>) -> Vec<ListenerNode> {
        self.seed_nodes_iter(runner)
            .map(|(id, _)| id.into())
            .collect()
    }

    async fn set_up_normal_nodes(&mut self, runner: &mut ClusterRunner<'_>) {
        eprintln!("setting up normal nodes: {}", self.config.normal_nodes);

        let node_config = RustNodeTestingConfig {
            max_peers: 100,
            initial_peers: self.seed_node_dial_addrs(runner),
            ..self.seed_config_async(runner).await
        };

        for _ in 0..(self.config.normal_nodes) {
            runner.add_rust_node(node_config.clone());
        }

        self.wait_for_all_nodes_synced(runner).await;
    }

    async fn set_up_snark_worker_nodes(&mut self, runner: &mut ClusterRunner<'_>) {
        eprintln!(
            "setting up rust snark worker nodes: {}",
            self.config.snark_workers
        );

        let node_config = RustNodeTestingConfig {
            max_peers: 100,
            initial_peers: self.seed_node_dial_addrs(runner),
            ..self.seed_config_async(runner).await
        };

        let bp_pub_keys = runner
            .nodes_iter()
            .filter_map(|(_, node)| {
                let sec_key = &node.config().block_producer.as_ref()?.sec_key;
                Some(sec_key.public_key())
            })
            .collect::<BTreeSet<_>>();

        let snarker_accounts = runner
            .accounts_with_sec_keys(ClusterNodeId::new_unchecked(0))
            .filter(|(sec_key, _)| !bp_pub_keys.contains(&sec_key.public_key()))
            .take(self.config.snark_workers)
            .collect::<Vec<_>>();

        for (sec_key, account) in snarker_accounts {
            eprintln!(
                "snark worker({}) balance: {} mina",
                sec_key.public_key(),
                account.balance.to_amount().as_u64()
            );
            let config = RustNodeTestingConfig {
                snark_worker: Some(SnarkerConfig {
                    public_key: sec_key.public_key(),
                    fee: CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        10_000_000.into(),
                    )),
                    strategy: SnarkerStrategy::Sequential,
                    auto_commit: true,
                    // TODO(binier): fix if we want to use real snarker.
                    path: "".into(),
                }),
                ..node_config.clone()
            };
            runner.add_rust_node(config);
        }

        self.wait_for_all_nodes_synced(runner).await;
    }

    async fn set_up_block_producer_nodes(&mut self, runner: &mut ClusterRunner<'_>) {
        let block_producers = runner.block_producer_sec_keys(ClusterNodeId::new_unchecked(0));

        assert!(self.config.block_producers <= block_producers.len());
        eprintln!(
            "setting up rust block producer nodes: {}/{}",
            self.config.block_producers,
            block_producers.len()
        );

        let node_config = RustNodeTestingConfig {
            max_peers: 100,
            initial_peers: self.seed_node_dial_addrs(runner),
            ..self.seed_config_async(runner).await
        };

        for (sec_key, stake) in block_producers
            .into_iter()
            .take(self.config.block_producers)
        {
            eprintln!(
                "block producer({}) stake: {stake} mina",
                sec_key.public_key()
            );
            let config = RustNodeTestingConfig {
                block_producer: Some(RustNodeBlockProducerTestingConfig {
                    config: BlockProducerConfig {
                        pub_key: sec_key.public_key().into(),
                        custom_coinbase_receiver: None,
                        proposed_protocol_version: None,
                    },
                    sec_key,
                }),
                ..node_config.clone()
            };
            runner.add_rust_node(config);
        }

        self.wait_for_all_nodes_synced(runner).await;
    }

    pub async fn run<'a>(&mut self, runner: &mut ClusterRunner<'a>) {
        self.set_up_seed_nodes(runner).await;
        self.set_up_normal_nodes(runner).await;
        self.set_up_snark_worker_nodes(runner).await;
        self.set_up_block_producer_nodes(runner).await;

        let run_until = self.config.run_until.clone();
        let advance_time = self.config.advance_time.clone();
        let start_t = redux::Instant::now();
        let mut last_printed_slot = 0;
        let virtual_initial_time = self.initial_time();

        while start_t.elapsed() < self.config.run_until_timeout {
            tokio::task::yield_now().await;
            let _ = runner
                .run(
                    RunCfg::default()
                        .advance_time(advance_time.clone())
                        .timeout(Duration::ZERO),
                )
                .await;

            let printed_elapsed_time = {
                let state = runner.nodes_iter().next().unwrap().1.state();
                if let Some(cur_slot) = state
                    .cur_global_slot()
                    .filter(|cur| *cur > last_printed_slot)
                {
                    let real_elapsed = start_t.elapsed();
                    let virtual_elapsed = state.time().checked_sub(virtual_initial_time).unwrap();
                    last_printed_slot = cur_slot;

                    eprintln!("[elapsed] real: {real_elapsed:?}, virtual: {virtual_elapsed:?}, global_slot: {cur_slot}");
                    true
                } else {
                    false
                }
            };

            if printed_elapsed_time {
                for (node_id, node) in runner.nodes_iter() {
                    let Some(best_tip) = node.state().transition_frontier.best_tip() else {
                        continue;
                    };
                    let consensus_state = &best_tip.header().protocol_state.body.consensus_state;

                    eprintln!(
                        "[node_status] node_{node_id} {} - {} [{}]; snarks: {}",
                        best_tip.height(),
                        best_tip.hash(),
                        best_tip.producer(),
                        best_tip.staged_ledger_diff().0.completed_works.len(),
                    );
                    let stop = match &run_until {
                        SimulatorRunUntil::Forever => false,
                        SimulatorRunUntil::Epoch(epoch) => {
                            consensus_state.epoch_count.as_u32() >= *epoch
                        }
                        SimulatorRunUntil::BlockchainLength(height) => best_tip.height() >= *height,
                    };
                    if stop {
                        return;
                    }
                }
            }
        }

        panic!("simulation timed out");
    }
}

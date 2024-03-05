mod config;
pub use config::*;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

use std::{collections::BTreeSet, time::Duration};

use node::{ActionKind, BlockProducerConfig, SnarkerConfig, SnarkerStrategy, State};
use rand::{Rng, SeedableRng};

use crate::{
    cluster::{ClusterNodeId, ClusterOcamlNodeId},
    node::{
        Node, OcamlNodeTestingConfig, OcamlStep, RustNodeBlockProducerTestingConfig,
        RustNodeTestingConfig,
    },
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{ClusterRunner, RunDecision},
};

pub struct Simulator {
    config: SimulatorConfig,
}

impl Simulator {
    pub fn new(config: SimulatorConfig) -> Self {
        Self { config }
    }

    fn seed_config(&self, runner: &ClusterRunner<'_>) -> RustNodeTestingConfig {
        let chain_id = runner
            .nodes_iter()
            .next()
            .map(|(_, node)| node.config().chain_id.clone())
            .unwrap_or_else(|| {
                runner
                    .ocaml_node(ClusterOcamlNodeId::new_unchecked(0))
                    .unwrap()
                    .chain_id()
                    .unwrap()
            });

        RustNodeTestingConfig {
            chain_id,
            // TODO(binier): make dynamic.
            // should match time in daemon_json
            initial_time: redux::Timestamp::new(1703494800000_000_000),
            max_peers: 1000,
            ask_initial_peers_interval: Duration::from_secs(60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: None,
            snark_worker: None,
            timeouts: Default::default(),
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
                .run(
                    Duration::from_secs(20),
                    |_, _, _| RunDecision::ContinueExec,
                    move |_, _, _, action| {
                        matches!(action.action().kind(), ActionKind::TransitionFrontierSynced)
                    },
                )
                .await
                .expect("error while waiting to sync genesis block from ocaml");
        }
        eprintln!("all rust nodes synced up");
    }

    async fn set_up_seed_nodes(&mut self, runner: &mut ClusterRunner<'_>) {
        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: runner
                .daemon_json_gen("2023-12-25T09:00:00Z", self.config.daemon_json.clone()),
        };

        let ocaml_node = runner.add_ocaml_node(ocaml_node_config);

        eprintln!("waiting for ocaml node readiness");
        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: ocaml_node,
                step: OcamlStep::WaitReady {
                    timeout: Duration::from_secs(5 * 60),
                },
            })
            .await
            .unwrap();

        eprintln!("setting up rust seed nodes: {}", self.config.seed_nodes);
        let seed_config = self.seed_config(runner);

        for _ in 0..(self.config.seed_nodes) {
            let rust_node = runner.add_rust_node(seed_config.clone());

            runner
                .exec_step(ScenarioStep::ConnectNodes {
                    dialer: rust_node,
                    listener: ListenerNode::Ocaml(ocaml_node),
                })
                .await
                .unwrap();
        }

        self.wait_for_all_nodes_synced(runner).await;

        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: ocaml_node,
                step: OcamlStep::KillAndRemove,
            })
            .await
            .unwrap();
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
            ..self.seed_config(runner)
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
            ..self.seed_config(runner)
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
                    public_key: sec_key.public_key().into(),
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
            ..self.seed_config(runner)
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

    pub async fn run<'a>(&mut self, mut runner: ClusterRunner<'a>) {
        self.set_up_seed_nodes(&mut runner).await;
        self.set_up_normal_nodes(&mut runner).await;
        self.set_up_snark_worker_nodes(&mut runner).await;
        self.set_up_block_producer_nodes(&mut runner).await;

        let run_until = self.config.run_until.clone();
        let mut timeout = self.config.run_until_timeout;
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        while !timeout.is_zero() {
            let t = redux::Instant::now();
            tokio::task::yield_now().await;
            let _ = runner
                .run(
                    Duration::ZERO,
                    |_, _, _| RunDecision::ContinueExec,
                    |_, _, _, _| false,
                )
                .await;

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
                match &run_until {
                    SimulatorRunUntil::Epoch(epoch) => {
                        let cur_epoch = consensus_state.epoch_count.as_u32();
                        if cur_epoch >= *epoch {
                            return;
                        }
                    }
                }
            }

            // advance global time randomly.
            let advance_time = Duration::from_millis(rng.gen_range(1..300));
            let elapsed = t.elapsed();
            let by_nanos = advance_time.as_nanos() as u64;
            eprintln!("[TIME] elapsed {elapsed:?}; advance_by: {advance_time:?}");
            runner
                .exec_step(ScenarioStep::AdvanceTime { by_nanos })
                .await
                .unwrap();

            timeout = timeout.saturating_sub(elapsed);
        }

        panic!("simulation timed out");
    }
}

use std::time::Duration;

use ledger::BaseLedger;
use mina_p2p_messages::v2;
use node::{transition_frontier::genesis::GenesisConfig, ActionKind};

use crate::{
    node::{DaemonJson, OcamlNodeTestingConfig, OcamlStep, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{ClusterRunner, RunCfg},
};

/// Block production basic test.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct BlockProductionBasic;

impl BlockProductionBasic {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let genesis_timestamp_ms = u64::from(initial_time) / 1_000_000;
        let genesis_cfg = GenesisConfig::Counts {
            whales: 2,
            fish: 0,
            constants: GenesisConfig::default_constants(genesis_timestamp_ms),
        };

        let (genesis_mask, _) = genesis_cfg.load().unwrap();

        let mut accounts = Vec::with_capacity(genesis_mask.num_accounts());
        genesis_mask.iter(|acc| {
            let pub_key = acc.public_key.into_address();
            let delegate = acc
                .delegate
                .as_ref()
                .map_or(pub_key.clone(), |pk| pk.into_address());
            let acc = serde_json::json!({
                "pk": pub_key,
                "balance": acc.balance.to_mina_str(),
                "delegate": delegate
            });
            // uncomment once json encoding is compatible
            // let acc = v2::MinaBaseAccountBinableArgStableV2::from(acc);
            accounts.push(acc);
        });

        let ocaml_node = runner.add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: DaemonJson::InMem(serde_json::json!({
                "genesis": {
                    "genesis_state_timestamp": node::core::log::to_rfc_3339(redux::Timestamp::new(genesis_timestamp_ms * 1_000_000)).unwrap(),
                },
                "ledger": {
                    "name": "custom",
                    "accounts": accounts,
                },
            })),
            block_producer: None,
        });
        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: ocaml_node,
                step: OcamlStep::WaitReady {
                    timeout: Duration::from_secs(5 * 60),
                },
            })
            .await
            .expect("ocaml node didn't become ready");
        let (chain_id, ocaml_node_peer_id) = {
            let node = runner.ocaml_node(ocaml_node).unwrap();
            let chain_id = node
                .chain_id()
                .expect("failed to get chain_id from ocaml node");
            (chain_id, node.peer_id())
        };

        let seed_cfg = RustNodeTestingConfig {
            chain_id: chain_id.into_bytes(),
            initial_time,
            genesis: genesis_cfg.into(),
            initial_peers: vec![ListenerNode::Ocaml(ocaml_node)],
            ..RustNodeTestingConfig::berkeley_default()
        };
        let seed_node = runner.add_rust_node(seed_cfg.clone());

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(60))
                    .action_handler(move |id, _, _, action| {
                        id == seed_node
                            && matches!(
                                action.action().kind(),
                                ActionKind::TransitionFrontierGenesisInject
                            )
                    }),
            )
            .await
            .expect("seed node didn't produce genesis block");

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(60))
                    .action_handler(move |id, state, _, action| {
                        dbg!(action.action().kind());
                        id == seed_node
                            && state
                                .p2p
                                .get_ready_peer(&ocaml_node_peer_id)
                                .map_or(false, |peer| peer.best_tip.is_some())
                    }),
            )
            .await
            .expect("seed node didn't produce genesis block");

        // make sure genesis blocks are the same.
        self.assert_all_nodes_best_tip_eq(&runner).await;
    }

    async fn assert_all_nodes_best_tip_eq(&self, runner: &ClusterRunner<'_>) {
        eprintln!("[check_tips]");
        for (id, node) in runner.ocaml_nodes_iter() {
            let tip = node
                .synced_best_tip_async()
                .await
                .expect("failed to get best tip from ocaml node")
                .expect("ocaml node not synced");
            eprintln!("{id:?} - {tip}");
        }
        for (id, node) in runner.nodes_iter() {
            let tip = node.state().transition_frontier.best_tip().unwrap().hash();
            eprintln!("{id:?} - {tip}");
        }
    }
}

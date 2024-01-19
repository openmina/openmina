use std::time::Duration;

use chrono::Timelike;

use crate::{
    node::{OcamlNodeTestingConfig, OcamlStep, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::{ClusterRunner, RunDecision},
};

/// Set up single Rust node and sync to custom genesis block/ledger.
///
/// Since we don't have a way to generate genesis block/ledger from
/// daemon.json directly, we start up ocaml node with that daemon.json,
/// connect to it and sync up from it for now.
///
/// 1. Start up ocaml node with custom genesis.
/// 2. Wait for ocaml node ready.
/// 3. Start rust node, connect to ocaml node and sync up from it.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeSyncToGenesisCustom;

impl SoloNodeSyncToGenesisCustom {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let now = chrono::Utc::now()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        eprintln!("Genesis timestamp: {now}");
        let initial_time =
            redux::Timestamp::new(now.timestamp_nanos_opt().unwrap().try_into().unwrap());

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: runner.daemon_json_load(
                "./tests/files/vrf_genesis_epoch/daemon.json".into(),
                &now.to_rfc3339(),
            ),
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

        let chain_id = runner.ocaml_node(ocaml_node).unwrap().chain_id().unwrap();

        // set here to be used in other child scenarios
        runner.set_chain_id(&chain_id);
        runner.set_initial_time(initial_time);

        let rust_node = runner.add_rust_node(RustNodeTestingConfig {
            chain_id,
            initial_time,
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(60 * 60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: None,
            snark_worker: None,
        });

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node,
                listener: ListenerNode::Ocaml(ocaml_node),
            })
            .await
            .unwrap();

        eprintln!("waiting for rust node to sync up from ocaml node");
        runner
            .run(
                Duration::from_secs(60),
                |_, _, _| RunDecision::ContinueExec,
                move |node_id, state, _, _| {
                    node_id == rust_node
                        && state.transition_frontier.sync.is_synced()
                        && state.transition_frontier.best_tip().is_some()
                },
            )
            .await
            .expect("error while waiting to sync genesis block from ocaml");
        eprintln!("rust node synced up from ocaml node");

        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: ocaml_node,
                step: OcamlStep::KillAndRemove,
            })
            .await
            .unwrap();
    }
}

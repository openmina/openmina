use std::{collections::BTreeMap, str::FromStr, time::Duration};

use node::{
    block_producer::{
        vrf_evaluator::VrfEvaluationOutputWithHash, BlockProducerEvent,
        BlockProducerVrfEvaluatorEvent,
    },
    event_source::Event,
    p2p::P2pTimeouts,
    BlockProducerConfig,
};
use openmina_node_account::AccountSecretKey;
use vrf::VrfEvaluationOutput;

use crate::{
    node::{OcamlVrfOutput, RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::{ClusterRunner, RunDecision},
};

/// Set up single Rust node and connect to an ocaml node with custom ledger and check if the node
/// yields the correct vrf outputs for each slot
///
/// 1. Run an ocaml node with custom deamon.json which contians a custom ledger
/// 2. Connect the Rust node to the ocaml node to get the genesis ledger and block
/// 3. Capture each BlockProducerVrfEvaluatorEvent and compare the output to the ocaml implementation
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeVrfGetCorrectSlots;

impl MultiNodeVrfGetCorrectSlots {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        eprintln!("Running vrf get correct ledgers scenario");

        let chain_id = runner.get_chain_id().unwrap();
        let initial_time = runner.get_initial_time().unwrap();

        let (initial_node, _) = runner.nodes_iter().last().unwrap();

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key_bs58 = "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9";
        let sec_key: AccountSecretKey = AccountSecretKey::from_str(sec_key_bs58).unwrap();

        let producer_node = runner.add_rust_node(RustNodeTestingConfig {
            chain_id,
            initial_time,
            genesis: node::BERKELEY_CONFIG.clone(),
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(60 * 60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: Some(RustNodeBlockProducerTestingConfig {
                config: BlockProducerConfig {
                    pub_key: sec_key.public_key().into(),
                    custom_coinbase_receiver: None,
                    proposed_protocol_version: None,
                },
                sec_key,
            }),
            snark_worker: None,
            timeouts: P2pTimeouts::default(),
            libp2p_port: None,
        });

        tokio::time::sleep(Duration::from_secs(2)).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: producer_node,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();

        eprintln!("Producer node connected");

        let file = std::fs::File::open("./tests/files/vrf_genesis_epoch/ocaml_vrf_1_to_7139.json")
            .unwrap();
        let loaded_ocaml_results: BTreeMap<u32, BTreeMap<u32, OcamlVrfOutput>> =
            serde_json::from_reader(file).unwrap();
        let outputs = LoadedOcamlVrfOutput(loaded_ocaml_results);

        runner
            .run(
                Duration::from_secs(400),
                |node_id, _, event| {
                    if node_id == producer_node {
                        if let Some(output) = capture_vrf_event(event) {
                            if output.evaluation_result.global_slot() >= 7140 {
                                return RunDecision::StopExec;
                            }
                            let ocaml_output =
                                outputs.get_won_slot(output.evaluation_result.global_slot());
                            compare_vrf(output, ocaml_output)
                        };
                    }
                    RunDecision::ContinueExec
                },
                |_, _, _, _| false,
            )
            .await
            .expect("Timeout - waiting for VRF evaluator to update producer and delegates");

        // ocaml_exec.kill_vrf_container();
    }
}

struct LoadedOcamlVrfOutput(BTreeMap<u32, BTreeMap<u32, OcamlVrfOutput>>);

impl LoadedOcamlVrfOutput {
    fn get_won_slot(&self, slot: u32) -> Option<&OcamlVrfOutput> {
        self.0
            .get(&slot)
            .and_then(|per_slot| per_slot.values().find(|res| res.threshold_met))
    }
}

fn capture_vrf_event(event: &Event) -> Option<&VrfEvaluationOutputWithHash> {
    let Event::BlockProducerEvent(BlockProducerEvent::VrfEvaluator(
        BlockProducerVrfEvaluatorEvent::Evaluated(output),
    )) = event
    else {
        return None;
    };

    Some(output)
}

fn compare_vrf(slot_result: &VrfEvaluationOutputWithHash, ocaml_result: Option<&OcamlVrfOutput>) {
    let global_slot = match &slot_result.evaluation_result {
        VrfEvaluationOutput::SlotWon(won_slot) => won_slot.global_slot,
        VrfEvaluationOutput::SlotLost(global_slot) => *global_slot,
    };

    eprintln!("Checking slot {}", global_slot);

    // Assertions
    if let Some(ocaml_won_slot) = ocaml_result {
        match &slot_result.evaluation_result {
            VrfEvaluationOutput::SlotWon(rust_won_slot) => {
                // assert_eq!(pk.to_string(), rust_won_slot.winner_account, "Winner account missmatch");
                assert_eq!(
                    ocaml_won_slot.vrf_output,
                    rust_won_slot.vrf_output.to_string(),
                    "VRF output missmatch"
                );
                assert_eq!(
                    ocaml_won_slot.vrf_output_fractional,
                    rust_won_slot.vrf_output.fractional(),
                    "Fractional missmatch"
                )
            }
            VrfEvaluationOutput::SlotLost(_) => panic!("Slot should have been won!"),
        }
    } else {
        assert!(
            matches!(
                slot_result.evaluation_result,
                VrfEvaluationOutput::SlotLost(_)
            ),
            "Slot shoud have been lost!"
        );
    }
}

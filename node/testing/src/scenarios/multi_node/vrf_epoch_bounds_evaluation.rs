use std::{str::FromStr, time::Duration};

use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use node::{p2p::P2pTimeouts, BlockProducerConfig, SnarkerConfig, SnarkerStrategy};
use openmina_node_account::AccountSecretKey;

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::ClusterRunner,
};

const GLOBAL_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const STEP_DURATION: Duration = Duration::from_secs(2);

const SECOND_EPOCH_LAST_SLOT: u32 = 14_279;
const THIRD_EPOCH_LAST_SLOT: u32 = 21_419;

/// TODO: DOCS
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeVrfEpochBoundsEvaluation;

impl MultiNodeVrfEpochBoundsEvaluation {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let chain_id = runner.get_chain_id().unwrap().into_bytes();
        let initial_time = runner.get_initial_time().unwrap();

        let (initial_node, _) = runner.nodes_iter().last().unwrap();

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key: AccountSecretKey =
            AccountSecretKey::from_str("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9")
                .unwrap();

        let rust_config = RustNodeTestingConfig {
            chain_id,
            initial_time,
            genesis: node::BERKELEY_CONFIG.clone(),
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(60 * 60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: None,
            snark_worker: None,
            timeouts: P2pTimeouts::default(),
            libp2p_port: None,
        };

        let producer_node = runner.add_rust_node(RustNodeTestingConfig {
            block_producer: Some(RustNodeBlockProducerTestingConfig {
                config: BlockProducerConfig {
                    pub_key: sec_key.public_key().into(),
                    custom_coinbase_receiver: None,
                    proposed_protocol_version: None,
                },
                sec_key: sec_key.clone(),
            }),
            ..rust_config.clone()
        });

        let snarker_node = runner.add_rust_node(RustNodeTestingConfig {
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
            ..rust_config
        });

        tokio::time::sleep(Duration::from_secs(2)).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: producer_node,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();

        eprintln!("[PREP] Producer node connected");

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: producer_node,
                listener: ListenerNode::Rust(snarker_node),
            })
            .await
            .unwrap();

        eprintln!("[PREP] Snarker node connected");

        // produce blocks until evaluation finishes for the first two epochs
        runner
            .produce_blocks_until(
                producer_node,
                "EPOCH 0-1 EVAL",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                false,
                |state, _, _| {
                    let last_evaluated_slot = state
                        .block_producer
                        .vrf_evaluator()
                        .unwrap()
                        .latest_evaluated_slot;

                    eprintln!("HIGHEST EVALUATED SLOT: {last_evaluated_slot}");

                    last_evaluated_slot == SECOND_EPOCH_LAST_SLOT
                },
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();
        let last_evaluated_slot = state
            .block_producer
            .vrf_evaluator()
            .unwrap()
            .latest_evaluated_slot;

        // should have all the slots in the first and second epoch
        assert_eq!(
            SECOND_EPOCH_LAST_SLOT, last_evaluated_slot,
            "VRF evaluation missing for second epoch"
        );

        // skip to the epoch bounds
        runner
            .advance_to_epoch_bounds(producer_node, GLOBAL_TIMEOUT, STEP_DURATION)
            .await;
        runner
            .produce_blocks_until(
                producer_node,
                "EPOCH 2 EVAL",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                false,
                |_, _, produced_blocks| produced_blocks >= 20,
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();

        let last_evaluated_slot = state
            .block_producer
            .vrf_evaluator()
            .unwrap()
            .latest_evaluated_slot;

        // after the epoch changes the new evaluation should start
        assert_ne!(
            SECOND_EPOCH_LAST_SLOT, last_evaluated_slot,
            "VRF evaluation not started for third epoch"
        );

        runner
            .produce_blocks_until(
                producer_node,
                "EPOCH 2 EVAL",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                false,
                |state, _, _| {
                    let last_evaluated_slot = state
                        .block_producer
                        .vrf_evaluator()
                        .unwrap()
                        .latest_evaluated_slot;

                    eprintln!("HIGHEST EVALUATED SLOT: {last_evaluated_slot}");

                    last_evaluated_slot == THIRD_EPOCH_LAST_SLOT
                },
            )
            .await;

        eprintln!("OK");
    }
}

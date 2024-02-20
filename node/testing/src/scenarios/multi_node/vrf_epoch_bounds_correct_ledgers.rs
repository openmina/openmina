use std::{str::FromStr, time::Duration};

use ledger::AccountIndex;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use node::account::AccountSecretKey;
use node::{
    block_producer::vrf_evaluator::{BlockProducerVrfEvaluatorStatus, EpochContext},
    p2p::P2pTimeouts,
    ActionKind, BlockProducerConfig, SnarkerConfig, SnarkerStrategy,
};

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{ClusterRunner, RunCfg},
};

const GLOBAL_TIMEOUT: Duration = Duration::from_secs(60 * 60);
// const STEP_DURATION: Duration = Duration::from_secs(60);
const STEP_DURATION: Duration = Duration::from_millis(500);

const SECOND_EPOCH_LAST_SLOT: u32 = 14_279;
const _THIRD_EPOCH_LAST_SLOT: u32 = 21_419;

const KEEP_SYNCED: bool = true;

/// TODO: DOCS
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeVrfEpochBoundsCorrectLedger;

impl MultiNodeVrfEpochBoundsCorrectLedger {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let start = tokio::time::Instant::now();
        let chain_id = runner.get_chain_id().unwrap().into_bytes();
        let initial_time = runner.get_initial_time().unwrap();

        let (initial_node, _) = runner.nodes_iter().last().unwrap();

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key: AccountSecretKey =
            AccountSecretKey::from_str("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9")
                .unwrap();

        // for account B62qrsEWVgwRMJatJzQCrepPisMF4QcB2PPBbu9pbodZRvxWM7ochkx
        let snark_worker =
            AccountSecretKey::from_str("EKFbTQSagw6vVaZyGWGZUE6GKKfMG8QTX5xoPPnXSeWhe2uaCCTD")
                .unwrap();

        let rust_config = RustNodeTestingConfig {
            chain_id,
            initial_time,
            genesis: node::config::BERKELEY_CONFIG.clone(),
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
                public_key: snark_worker.public_key(),
                fee: CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                    0.into(),
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

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();
        let producer_peer_id = state.p2p.my_id().to_string();
        eprintln!("[PREP] Producer node connected, peer_id: {producer_peer_id}");

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: producer_node,
                listener: ListenerNode::Rust(snarker_node),
            })
            .await
            .unwrap();

        eprintln!("[PREP] Snarker node connected");

        let mut total_produced_blocks: u32 = 0;

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(5))
                    .action_handler(move |node_id, _, _, action| {
                        if node_id == producer_node {
                            matches!(
                                action.action().kind(),
                                ActionKind::BlockProducerVrfEvaluatorBeginEpochEvaluation
                            )
                        } else {
                            false
                        }
                    }),
            )
            .await
            .unwrap();

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();

        let vrf_evaluator = &state
            .block_producer
            .vrf_evaluator()
            .expect("No Vrf evaluator");

        let initial_balance = if let Some(pending_evaluation) = vrf_evaluator.current_evaluation() {
            let (_, balance) = pending_evaluation
                .epoch_data
                .delegator_table
                .get(&AccountIndex(1))
                .expect("Account not found");
            eprintln!("Initial balance: {balance}");
            *balance
        } else {
            panic!("No pending evaluation!");
        };

        // produce blocks until evaluation finishes for the first two epochs
        total_produced_blocks += runner
            .produce_blocks_until(
                producer_node,
                "COLLECTING BLOCKS",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                KEEP_SYNCED,
                |state, _, produced_blocks| {
                    let vrf_evaluator_status =
                        &state.block_producer.vrf_evaluator().unwrap().status;

                    eprintln!("Evaluator state: {}", vrf_evaluator_status);
                    // TODO: remove
                    produced_blocks >= 3600
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
        total_produced_blocks += runner
            .advance_to_epoch_bounds(producer_node, GLOBAL_TIMEOUT, STEP_DURATION)
            .await;
        total_produced_blocks += runner
            .produce_blocks_until(
                producer_node,
                "NEW EPOCH",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                KEEP_SYNCED,
                |state, _, produced_blocks| {
                    eprintln!("\nSnarks: {}", state.snark_pool.last_index());
                    eprintln!("Produced blocks: {produced_blocks}\n");
                    produced_blocks >= 290
                },
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();

        let vrf_evaluator_state = &state.block_producer.vrf_evaluator().unwrap();
        if let BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { .. } =
            vrf_evaluator_state.status
        {
            if matches!(vrf_evaluator_state.epoch_context(), EpochContext::Next(_)) {
                panic!("Evaluator evaluating next epoch. Should wait 290 blocks!");
            }
        }

        total_produced_blocks += runner
            .produce_blocks_until(
                producer_node,
                "NEW EPOCH 2",
                GLOBAL_TIMEOUT,
                STEP_DURATION,
                KEEP_SYNCED,
                |state, _, produced_blocks| {
                    eprintln!("\nSnarks: {}", state.snark_pool.last_index());
                    eprintln!("Produced blocks: {produced_blocks}\n");
                    produced_blocks >= 2
                },
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node, false).unwrap();

        let vrf_evaluator = &state
            .block_producer
            .vrf_evaluator()
            .expect("No Vrf evaluator");

        let new_balance = if let Some(pending_evaluation) = vrf_evaluator.current_evaluation() {
            let (_, balance) = pending_evaluation
                .epoch_data
                .delegator_table
                .get(&AccountIndex(1))
                .expect("Account not found");
            eprintln!("New balance: {balance}");
            *balance
        } else {
            panic!("No pending evaluation!");
        };

        let vrf_evaluator_state = &state.block_producer.vrf_evaluator().unwrap();
        if let BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { .. } =
            vrf_evaluator_state.status
        {
            if matches!(vrf_evaluator_state.epoch_context(), EpochContext::Next(_)) {
                eprintln!("Evaluator correctly started evaluating the next epoch");
            } else {
                panic!("Evaluator should have started the evaluation at this point!");
            }
        }

        let expected_balance = (total_produced_blocks * 720_000_000) as u64 + initial_balance;

        assert_eq!(expected_balance, new_balance);

        eprintln!("Test duration: {:?}", start.elapsed());

        eprintln!("OK");
    }
}

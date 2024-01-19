use std::{str::FromStr, time::Duration};

use ledger::AccountIndex;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use node::{account::AccountSecretKey, BlockProducerConfig, SnarkerConfig, SnarkerStrategy, ActionKind};

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{cluster_runner::ClusterRunner, RunDecision},
};

const GLOBAL_TIMEOUT: Duration = Duration::from_secs(60 * 60);

const SECOND_EPOCH_LAST_SLOT: u32 = 14_279;
const THIRD_EPOCH_LAST_SLOT: u32 = 21_419;

/// TODO: DOCS
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeVrfEpochBoundsCorrectLedger;

impl MultiNodeVrfEpochBoundsCorrectLedger {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let chain_id = runner.get_chain_id().unwrap();
        let initial_time = runner.get_initial_time().unwrap();

        let (initial_node, _) = runner.nodes_iter().last().unwrap();

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key: AccountSecretKey =
            AccountSecretKey::from_str("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9")
                .unwrap();
        
        // for account B62qrsEWVgwRMJatJzQCrepPisMF4QcB2PPBbu9pbodZRvxWM7ochkx
        let snark_worker = AccountSecretKey::from_str("EKFbTQSagw6vVaZyGWGZUE6GKKfMG8QTX5xoPPnXSeWhe2uaCCTD").unwrap();

        let rust_config = RustNodeTestingConfig {
            chain_id,
            initial_time,
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(60 * 60),
            initial_peers: Vec::new(),
            peer_id: Default::default(),
            block_producer: None,
            snark_worker: None,
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

        eprintln!("[PREP] Producer node connected");

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: producer_node,
                listener: ListenerNode::Rust(snarker_node),
            })
            .await
            .unwrap();

        eprintln!("[PREP] Snarker node connected");

        let mut total_produced_blocks: u32 = 0;

        // total_produced_blocks += runner
        //     .produce_blocks_until(
        //         producer_node,
        //         "COLLECTING BLOCKS",
        //         GLOBAL_TIMEOUT,
        //         |state, _, produced_blocks| {
        //             produced_blocks >= 1
        //         },
        //     )
        //     .await;
        runner.run(Duration::from_secs(5), |_, _, _| RunDecision::ContinueExec, move |node_id, _, _, action| {
            if node_id == producer_node {
                matches!(action.action().kind(), ActionKind::BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccess)
            } else {
                false
            }
        }).await
        .unwrap();

        let (state, _) = runner.node_pending_events(producer_node).unwrap();

        // TODO: Delete
        let (pk, old_balance) = state
            .block_producer
            .vrf_evaluator()
            .unwrap()
            .next_epoch_data
            .clone()
            .unwrap()
            .delegator_table
            .get(&AccountIndex(1))
            .unwrap()
            .clone();

        eprintln!();
        eprintln!("{pk}: {old_balance}");
        eprintln!();

        // produce blocks until evaluation finishes for the first two epochs
        total_produced_blocks += runner
            .produce_blocks_until(
                producer_node,
                "COLLECTING BLOCKS",
                GLOBAL_TIMEOUT,
                |state, _, produced_blocks| {
                    eprintln!("\nSnarks: {}", state.snark_pool.last_index());
                    eprintln!("Produced blocks: {produced_blocks}\n");
                    produced_blocks >= 3400
                },
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node).unwrap();
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
            .advance_to_epoch_bounds(producer_node, GLOBAL_TIMEOUT)
            .await;
        total_produced_blocks += runner
            .produce_blocks_until(
                producer_node,
                "NEW EPOCH",
                GLOBAL_TIMEOUT,
                |state, _, produced_blocks| {
                    eprintln!("\nSnarks: {}", state.snark_pool.last_index());
                    eprintln!("Produced blocks: {produced_blocks}\n");
                    produced_blocks >= 2
                },
            )
            .await;

        let (state, _) = runner.node_pending_events(producer_node).unwrap();

        // TODO: Delete
        let (pk, new_balance) = state
            .block_producer
            .vrf_evaluator()
            .unwrap()
            .next_epoch_data
            .clone()
            .unwrap()
            .delegator_table
            .get(&AccountIndex(1))
            .unwrap()
            .clone();

        eprintln!();
        eprintln!("{pk}: {new_balance}");
        eprintln!();

        eprintln!("OK");
    }
}

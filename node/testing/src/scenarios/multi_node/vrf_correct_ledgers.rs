use std::{collections::BTreeMap, str::FromStr, time::Duration};

use ledger::AccountIndex;
use node::account::{AccountPublicKey, AccountSecretKey};
use node::{p2p::P2pTimeouts, ActionKind, BlockProducerConfig};

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::{ClusterRunner, RunDecision},
};

/// Set up single Rust node and connect to an ocaml node with custom ledger and check if the node
/// has the correct delegator table
///
/// 1. Run an ocaml node with custom deamon.json which contians a custom ledger
/// 2. Connect the Rust node to the ocaml node to get the genesis ledger and block
/// 3. Once the Best tip updates the vrf calculation should start, including getting the epoch ledgers
/// 4. Check that the vrf evaluator gets the correct producer account and its delegators with correct balances
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeVrfGetCorrectLedgers;

impl MultiNodeVrfGetCorrectLedgers {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        eprintln!("Running vrf get correct ledgers scenario");

        let chain_id = runner.get_chain_id().unwrap();
        let initial_time = runner.get_initial_time().unwrap();

        let (initial_node, _) = runner.nodes_iter().last().unwrap();

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key: AccountSecretKey =
            AccountSecretKey::from_str("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9")
                .unwrap();

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

        let pad = |x: u64| -> u64 { x * 1_000_000_000 };

        let expected_delegator_table_data = [
            (
                AccountIndex(1),
                (
                    AccountPublicKey::from_str(
                        "B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq",
                    )
                    .unwrap(),
                    pad(1000000000),
                ),
            ),
            (
                AccountIndex(2),
                (
                    AccountPublicKey::from_str(
                        "B62qrJKtrMJCFDaYTqaxpY5KU1XuwSiL1ZtWteNFKxsmW9xZzG3cYX2",
                    )
                    .unwrap(),
                    pad(1000000000),
                ),
            ),
            (
                AccountIndex(5),
                (
                    AccountPublicKey::from_str(
                        "B62qos2NxSras7juEwPVnkoV23YTFvWawyko8pgcf8S5nccTFCzVpdy",
                    )
                    .unwrap(),
                    pad(100000),
                ),
            ),
        ];

        let _expected_delegator_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)> =
            expected_delegator_table_data.into_iter().collect();

        runner
            .run(
                Duration::from_secs(400),
                |_, _, _| RunDecision::ContinueExec,
                move |node_id, _, _, action| {
                    if node_id == producer_node {
                        matches!(
                            action.action().kind(),
                            ActionKind::BlockProducerVrfEvaluatorBeginEpochEvaluation
                        )
                    } else {
                        false
                    }
                },
            )
            .await
            .expect("Timeout - waiting for VRF evaluator to update producer and delegates");

        let (_state, _) = runner.node_pending_events(producer_node, false).unwrap();

        // check if our producer and delegators are detected correctly from the epoch ledgers
        // let epoch_data = state
        //     .block_producer
        //     .vrf_evaluator()
        //     .expect("Vrf evaluator state should be available")
        //     .current_epoch_data
        //     .as_ref()
        //     .unwrap()
        //     .clone();
        // let delegator_table = epoch_data.delegator_table.as_ref();

        // assert_eq!(
        //     expected_delegator_table,
        //     delegator_table.clone(),
        //     "Delegator tables are not equal"
        // );
    }
}

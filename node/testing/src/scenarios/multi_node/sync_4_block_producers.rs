use std::time::Duration;

use node::BlockProducerConfig;

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::ClusterRunner,
};

/// Create and Sync up 4 block producer nodes.
///
/// 1. Create 4 block producer nodes.
/// 2. Connect them to each other along with the initial node.
/// 3. Wait for all nodes to be synced.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeSync4BlockProducers;

impl MultiNodeSync4BlockProducers {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let (initial_node, node_config) = runner
            .nodes_iter()
            .next()
            .map(|(id, node)| (id, node.config().clone()))
            .unwrap();

        let mut block_producers = runner.block_producer_sec_keys(initial_node);

        let [whale1, whale2, fish1, fish2] = std::array::from_fn(|_| {
            let (sec_key, _) = block_producers.pop().unwrap();
            runner.add_rust_node(RustNodeTestingConfig {
                block_producer: Some(RustNodeBlockProducerTestingConfig {
                    config: BlockProducerConfig {
                        pub_key: sec_key.public_key().into(),
                        custom_coinbase_receiver: None,
                        proposed_protocol_version: None,
                    },
                    sec_key,
                }),
                ..node_config.clone()
            })
        });

        // TODO(binier): proper way to wait for all nodes to be ready.
        tokio::time::sleep(Duration::from_secs(2)).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: whale1,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: whale2,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: fish1,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: fish2,
                listener: ListenerNode::Rust(initial_node),
            })
            .await
            .unwrap();

        eprintln!("waiting for 4 block producer nodes to sync up.");
        runner
            .run_until_nodes_synced(Duration::from_secs(3 * 60), &[whale1, whale2, fish1, fish2])
            .await
            .unwrap();
        eprintln!("4 block producer nodes create and synced up.");
    }
}

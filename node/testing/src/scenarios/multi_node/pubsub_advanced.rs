use std::{iter, str::FromStr, time::Duration};

use node::{account::AccountSecretKey, BlockProducerConfig};

use crate::{
    node::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig},
    scenario::ListenerNode,
    scenarios::ClusterRunner,
};

/// Create and Sync up 50 nodes, one amoung them is block producer.
///
/// 1. Create the nodes.
/// 2. Connect them to each other.
/// 3. Wait kademlia bootstrap is done, observe the connection graph.
/// 4. Wait pubsub mesh construction is done, observe the mesh.
/// 5. Wait block is produced and observe the propagation.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodePubsubPropagateBlock;

impl MultiNodePubsubPropagateBlock {
    const WORKERS: usize = 4;

    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        // let seed_node = ListenerNode::Custom(
        //     "/ip4/34.135.63.47/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag"
        //         .parse()
        //         .unwrap(),
        // );

        let seed_config = RustNodeTestingConfig::devnet_default()
            .max_peers(100)
            .ask_initial_peers_interval(Duration::from_secs(60 * 60));

        let seed_node = ListenerNode::Rust(runner.add_rust_node(seed_config));

        // for account B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq
        let sec_key =
            AccountSecretKey::from_str("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9")
                .unwrap();

        let producer_node = runner.add_rust_node(RustNodeTestingConfig {
            initial_time: redux::Timestamp::ZERO,
            genesis: node::config::DEVNET_CONFIG.clone(),
            max_peers: 10,
            ask_initial_peers_interval: Duration::from_secs(60 * 60),
            initial_peers: vec![seed_node.clone()],
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
            timeouts: Default::default(),
            libp2p_port: None,
            recorder: Default::default(),
        });

        tokio::time::sleep(Duration::from_secs(2)).await;

        eprintln!("Producer node connected");

        let worker_config = RustNodeTestingConfig::devnet_default()
            .initial_peers(vec![seed_node])
            .max_peers(10)
            .ask_initial_peers_interval(Duration::from_secs(60 * 60));
        let workers = iter::repeat(worker_config)
            .take(Self::WORKERS)
            .map(|config| runner.add_rust_node(config))
            .collect::<Vec<_>>();

        runner
            .run_until_nodes_synced(Duration::from_secs(10 * 60), &workers)
            .await
            .unwrap();

        let _ = producer_node;
        // TODO:
    }
}

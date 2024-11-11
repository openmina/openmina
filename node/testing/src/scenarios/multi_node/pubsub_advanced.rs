use std::{
    str,
    sync::{Arc, Mutex},
    time::Duration,
};

use mina_p2p_messages::{binprot::BinProtRead, gossip, v2};
use node::{
    p2p::{P2pNetworkAction, P2pNetworkPubsubAction, PeerId},
    transition_frontier::genesis::{GenesisConfig, NonStakers},
    Action, ActionWithMeta, P2pAction,
};

use crate::{
    node::Recorder,
    scenarios::{ClusterRunner, RunCfgAdvanceTime},
    service::NodeTestingService,
    simulator::{Simulator, SimulatorConfig, SimulatorRunUntil},
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
    const WORKERS: usize = 10;

    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let graph = Arc::new(Mutex::new("digraph {\n".to_owned()));
        let factory = || {
            let graph = graph.clone();
            move |_id,
                  state: &node::State,
                  _service: &NodeTestingService,
                  action: &ActionWithMeta| {
                let this = state.p2p.my_id();

                let cut = |peer_id: &PeerId| {
                    let st = peer_id.to_string();
                    let len = st.len();
                    st[(len - 6)..len].to_owned()
                };
                let this = cut(&this);

                match action.action() {
                    Action::P2p(P2pAction::Network(P2pNetworkAction::Pubsub(
                        P2pNetworkPubsubAction::OutgoingMessage { msg, peer_id },
                    ))) => {
                        for publish_message in &msg.publish {
                            let mut slice = &publish_message.data()[8..];
                            if let Ok(gossip::GossipNetMessageV2::NewState(block)) =
                                gossip::GossipNetMessageV2::binprot_read(&mut slice)
                            {
                                let height = block
                                    .header
                                    .protocol_state
                                    .body
                                    .consensus_state
                                    .global_slot();
                                let mut lock = graph.lock().unwrap();
                                *lock = format!(
                                    "{lock}  \"{this}\" -> \"{}\" [label=\"{height}\"];\n",
                                    cut(peer_id)
                                );
                            }
                        }
                        false
                    }
                    _ => false,
                }
            }
        };

        let initial_time = redux::Timestamp::global_now();
        let mut constants = v2::PROTOCOL_CONSTANTS.clone();
        constants.genesis_state_timestamp =
            v2::BlockTimeTimeStableV1((u64::from(initial_time) / 1_000_000).into());
        let genesis_cfg = GenesisConfig::Counts {
            whales: 1,
            fish: 0,
            non_stakers: NonStakers::None,
            constants,
        };
        let config = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 1,
            normal_nodes: Self::WORKERS,
            snark_workers: 1,
            block_producers: 1,
            advance_time: RunCfgAdvanceTime::Rand(1..=200),
            run_until: SimulatorRunUntil::BlockchainLength(4),
            run_until_timeout: Duration::from_secs(10 * 60),
            recorder: Recorder::StateWithInputActions,
        };
        let mut simulator = Simulator::new(initial_time, config);
        simulator
            .setup_and_run_with_listener(&mut runner, factory)
            .await;

        println!("{}}}\n", graph.lock().unwrap());
    }
}

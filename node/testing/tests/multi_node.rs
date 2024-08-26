use openmina_node_testing::scenarios::multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining;
#[cfg(not(feature = "p2p-webrtc"))]
use openmina_node_testing::scenarios::multi_node::basic_connectivity_peer_discovery::MultiNodeBasicConnectivityPeerDiscovery;

mod common;

#[cfg(not(feature = "p2p-webrtc"))]
scenario_test!(
    peer_discovery,
    MultiNodeBasicConnectivityPeerDiscovery,
    MultiNodeBasicConnectivityPeerDiscovery
);

scenario_test!(
    initial_joining,
    MultiNodeBasicConnectivityInitialJoining,
    MultiNodeBasicConnectivityInitialJoining
);

#[cfg(feature = "p2p-libp2p")]
scenario_test!(
    propagate_block,
    openmina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock,
    openmina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock
);

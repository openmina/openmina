use openmina_node_testing::scenarios::p2p::basic_outgoing_connections::{
    ConnectToInitialPeers, ConnectToInitialPeersBecomeReady, ConnectToUnavailableInitialPeers,
    DontConnectToInitialPeerWithSameId, DontConnectToNodeWithSameId, DontConnectToSelfInitialPeer,
    MakeMultipleOutgoingConnections, MakeOutgoingConnection,
};

mod common;

scenario_test!(
    make_connection,
    MakeOutgoingConnection,
    MakeOutgoingConnection,
    true
);
scenario_test!(
    make_multiple_connections,
    MakeMultipleOutgoingConnections,
    MakeMultipleOutgoingConnections,
    true
);

scenario_test!(
    dont_connect_to_node_same_id,
    DontConnectToNodeWithSameId,
    DontConnectToNodeWithSameId,
    true
);
scenario_test!(
    dont_connect_to_initial_peer_same_id,
    DontConnectToInitialPeerWithSameId,
    DontConnectToInitialPeerWithSameId,
    true
);
scenario_test!(
    dont_connect_to_self_initial_peer,
    DontConnectToSelfInitialPeer,
    DontConnectToSelfInitialPeer
);

scenario_test!(
    connect_to_all_initial_peers,
    ConnectToInitialPeers,
    ConnectToInitialPeers,
    true
);
scenario_test!(
    connect_to_offline_initial_peers,
    ConnectToUnavailableInitialPeers,
    ConnectToUnavailableInitialPeers,
    true
);
scenario_test!(
    connect_to_all_initial_peers_become_ready,
    ConnectToInitialPeersBecomeReady,
    ConnectToInitialPeersBecomeReady,
    true
);

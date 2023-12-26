use openmina_node_testing::scenarios::p2p::basic_connection_handling::{
    AllNodesConnectionsAreSymmetric, ConnectionStability, SeedConnectionsAreSymmetric,
    SimultaneousConnections,
};

mod common;

scenario_test!(
    simultaneous_connections,
    SimultaneousConnections,
    SimultaneousConnections
);
scenario_test!(
    all_nodes_connections_are_symmetric,
    AllNodesConnectionsAreSymmetric,
    AllNodesConnectionsAreSymmetric
);
scenario_test!(
    seed_connections_are_symmetric,
    SeedConnectionsAreSymmetric,
    SeedConnectionsAreSymmetric
);

// ignore
// scenario_test!(max_number_of_peers, MaxNumberOfPeers, MaxNumberOfPeers);

scenario_test!(
    connection_stability,
    ConnectionStability,
    ConnectionStability
);

use mina_node_testing::scenarios::p2p::basic_connection_handling::{
    AllNodesConnectionsAreSymmetric, MaxNumberOfPeersIncoming, MaxNumberOfPeersIs1,
    SeedConnectionsAreSymmetric, SimultaneousConnections,
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

scenario_test!(
    max_number_of_peers_incoming,
    MaxNumberOfPeersIncoming,
    MaxNumberOfPeersIncoming
);

scenario_test!(
    max_number_of_peers_is_one,
    MaxNumberOfPeersIs1,
    MaxNumberOfPeersIs1
);

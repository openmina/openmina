use openmina_node_testing::scenarios::p2p::basic_connection_handling::{
    AllNodesConnectionsAreSymmetric, MaxNumberOfPeersIncoming, MaxNumberOfPeersIs1,
    SeedConnectionsAreSymmetric, SimultaneousConnections,
};

mod common;

// TODO: test fails spuriously because of connection error
scenario_test!(
    simultaneous_connections,
    SimultaneousConnections,
    SimultaneousConnections
);

// TODO: test fails because it keeps on running Kademlia::Init
scenario_test!(
    all_nodes_connections_are_symmetric,
    AllNodesConnectionsAreSymmetric,
    AllNodesConnectionsAreSymmetric
);

// TODO: test fails because it keeps on running Kademlia::Init
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

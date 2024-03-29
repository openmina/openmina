use openmina_node_testing::scenarios::p2p::basic_connection_handling::{
    AllNodesConnectionsAreSymmetric, MaxNumberOfPeers, MaxNumberOfPeersIs1,
    SeedConnectionsAreSymmetric, SimultaneousConnections,
};

mod common;

// TODO: test fails spuriously because of connection error
scenario_test!(
    #[ignore = "fails spuriously because of connection error"]
    simultaneous_connections,
    SimultaneousConnections,
    SimultaneousConnections
);

// TODO: test fails because it keeps on running Kademlia::Init
scenario_test!(
    #[ignore = "peers randomly disconnect, probably because of kademlia"]
    all_nodes_connections_are_symmetric,
    AllNodesConnectionsAreSymmetric,
    AllNodesConnectionsAreSymmetric
);

// TODO: test fails because it keeps on running Kademlia::Init
scenario_test!(
    #[ignore = "peers randomly disconnect, probably because of kademlia"]
    seed_connections_are_symmetric,
    SeedConnectionsAreSymmetric,
    SeedConnectionsAreSymmetric
);

scenario_test!(max_number_of_peers, MaxNumberOfPeers, MaxNumberOfPeers);

scenario_test!(
    max_number_of_peers_is_one,
    MaxNumberOfPeersIs1,
    MaxNumberOfPeersIs1
);

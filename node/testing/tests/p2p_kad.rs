use openmina_node_testing::scenarios::p2p::kademlia::{IncomingFindNode, KademliaBootstrap};

mod common;

scenario_test!(
    #[ignore = "Needs to be updated"]
    incoming_find_node,
    IncomingFindNode,
    IncomingFindNode
);

scenario_test!(
    #[ignore = "Needs to be updated"]
    kademlia_bootstrap,
    KademliaBootstrap,
    KademliaBootstrap
);

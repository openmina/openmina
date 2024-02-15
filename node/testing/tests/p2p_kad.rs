use openmina_node_testing::scenarios::p2p::kademlia::{IncomingFindNode, KademliaBootstrap};

mod common;

scenario_test!(
    incoming_find_node,
    IncomingFindNode,
    IncomingFindNode
);

scenario_test!(
    kademlia_bootstrap,
    KademliaBootstrap,
    KademliaBootstrap
);

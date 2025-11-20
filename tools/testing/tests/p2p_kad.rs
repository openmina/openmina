use mina_node_testing::scenarios::p2p::kademlia::KademliaBootstrap;

mod common;

scenario_test!(kademlia_bootstrap, KademliaBootstrap, KademliaBootstrap);

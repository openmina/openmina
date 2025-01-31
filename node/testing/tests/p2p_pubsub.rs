use openmina_node_testing::scenarios::p2p::pubsub::P2pReceiveMessage;

mod common;

scenario_test!(pubsub_receive_block, P2pReceiveMessage, P2pReceiveMessage);

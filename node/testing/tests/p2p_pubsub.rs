use openmina_node_testing::scenarios::p2p::pubsub::P2pReceiveBlock;

mod common;

// TODO: test fails spuriously because of connection error
scenario_test!(pubsub_receive_block, P2pReceiveBlock, P2pReceiveBlock);

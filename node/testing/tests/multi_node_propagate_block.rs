mod common;

#[cfg(feature = "p2p-libp2p")]
scenario_test!(
    propagate_block,
    openmina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock,
    openmina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock
);

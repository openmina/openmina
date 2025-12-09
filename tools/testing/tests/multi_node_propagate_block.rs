mod common;

#[cfg(feature = "p2p-libp2p")]
scenario_test!(
    propagate_block,
    mina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock,
    mina_node_testing::scenarios::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock
);

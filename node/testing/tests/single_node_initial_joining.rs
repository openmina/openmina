use mina_node_testing::scenarios::solo_node::basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining;

mod common;

scenario_test!(
    initial_joining,
    SoloNodeBasicConnectivityInitialJoining,
    SoloNodeBasicConnectivityInitialJoining
);

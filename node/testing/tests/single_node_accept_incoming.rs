#![cfg(not(feature = "p2p-webrtc"))]

use mina_node_testing::scenarios::solo_node::basic_connectivity_accept_incoming::SoloNodeBasicConnectivityAcceptIncoming;

mod common;

scenario_test!(
    accept_incoming,
    SoloNodeBasicConnectivityAcceptIncoming,
    SoloNodeBasicConnectivityAcceptIncoming
);

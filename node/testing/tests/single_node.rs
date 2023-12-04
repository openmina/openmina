#[cfg(not(feature = "p2p-webrtc"))]
use openmina_node_testing::scenarios::solo_node::basic_connectivity_accept_incoming::SoloNodeBasicConnectivityAcceptIncoming;
use openmina_node_testing::scenarios::solo_node::{
    basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining,
    sync_root_snarked_ledger::SoloNodeSyncRootSnarkedLedger,
};

mod common;

#[cfg(not(feature = "p2p-webrtc"))]
scenario_test!(
    accept_incoming,
    SoloNodeBasicConnectivityAcceptIncoming,
    SoloNodeBasicConnectivityAcceptIncoming
);

scenario_test!(
    initial_joining,
    SoloNodeBasicConnectivityInitialJoining,
    SoloNodeBasicConnectivityInitialJoining
);

scenario_test!(
    sync_root_snarked_ledger,
    SoloNodeSyncRootSnarkedLedger,
    SoloNodeSyncRootSnarkedLedger
);

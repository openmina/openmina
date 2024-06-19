#[cfg(not(feature = "p2p-webrtc"))]
use openmina_node_testing::scenarios::solo_node::basic_connectivity_accept_incoming::SoloNodeBasicConnectivityAcceptIncoming;
use openmina_node_testing::scenarios::solo_node::{
    basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining,
    // bootstrap::SoloNodeBootstrap ,
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
    #[ignore = "investigate falure"]
    initial_joining,
    SoloNodeBasicConnectivityInitialJoining,
    SoloNodeBasicConnectivityInitialJoining
);

scenario_test!(
    #[ignore = "investigate falure"]
    sync_root_snarked_ledger,
    SoloNodeSyncRootSnarkedLedger,
    SoloNodeSyncRootSnarkedLedger
);

// TODO: re-enable after #506 has been solved
// scenario_test!(
//     bootstrap_from_replayer,
//     SoloNodeBootstrap,
//     SoloNodeBootstrap
// );

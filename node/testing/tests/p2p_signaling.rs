#![cfg(feature = "p2p-webrtc")]

use openmina_node_testing::scenarios::p2p::signaling::P2pSignaling;

mod common;

scenario_test!(p2p_signaling, P2pSignaling, P2pSignaling, true);

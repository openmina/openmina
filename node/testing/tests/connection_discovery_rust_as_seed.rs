#![cfg(all(not(feature = "p2p-webrtc"), feature = "p2p-libp2p"))]

use openmina_node_testing::scenarios::multi_node::connection_discovery::RustNodeAsSeed;

mod common;

scenario_test!(rust_as_seed, RustNodeAsSeed, RustNodeAsSeed);

#![cfg(all(not(feature = "p2p-webrtc"), feature = "p2p-libp2p"))]

use openmina_node_testing::scenarios::multi_node::connection_discovery::RustToOCamlViaSeed;

mod common;

scenario_test!(
    rust_to_ocaml_via_seed,
    RustToOCamlViaSeed,
    RustToOCamlViaSeed
);

#![cfg(all(not(feature = "p2p-webrtc"), feature = "p2p-libp2p"))]

use openmina_node_testing::scenarios::multi_node::connection_discovery::OCamlToRustViaSeed;

mod common;

scenario_test!(
    ocaml_to_rust_via_seed,
    OCamlToRustViaSeed,
    OCamlToRustViaSeed
);

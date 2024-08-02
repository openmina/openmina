#![cfg(all(not(feature = "p2p-webrtc"), feature = "p2p-libp2p"))]

use openmina_node_testing::scenarios::multi_node::connection_discovery::OCamlToRust;

mod common;

scenario_test!(ocaml_to_rust, OCamlToRust, OCamlToRust);

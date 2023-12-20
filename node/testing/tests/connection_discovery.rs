#![cfg(all(not(feature = "p2p-webrtc"), feature = "p2p-libp2p"))]

use openmina_node_testing::scenarios::multi_node::connection_discovery::{
    OCamlToRust, OCamlToRustViaSeed, RustNodeAsSeed, RustToOCaml, RustToOCamlViaSeed,
};

mod common;

scenario_test!(rust_to_ocaml, RustToOCaml, RustToOCaml);
scenario_test!(ocaml_to_rust, OCamlToRust, OCamlToRust);
scenario_test!(
    rust_to_ocaml_via_seed,
    RustToOCamlViaSeed,
    RustToOCamlViaSeed
);
scenario_test!(
    ocaml_to_rust_via_seed,
    OCamlToRustViaSeed,
    OCamlToRustViaSeed
);
scenario_test!(rust_as_seed, RustNodeAsSeed, RustNodeAsSeed);

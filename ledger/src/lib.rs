#![allow(dead_code)]

// We need a feature to tests both nodejs and browser
// https://github.com/rustwasm/wasm-bindgen/issues/2571
#[cfg(not(feature = "in_nodejs"))]
#[cfg(target_family = "wasm")]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

#[cfg(not(target_family = "wasm"))]
mod ffi;

mod account;
mod address;
mod base;
mod hash;
mod mask;
mod poseidon;
mod tree;
mod tree_version;
mod util;

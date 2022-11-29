#[cfg(target_family = "wasm")]
#[cfg(test)]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

#[cfg(target_family = "wasm")]
pub mod rayon;

mod block;
pub mod hash;
mod public_input;
pub mod utils;

pub use block::{
    accumulator_check::{accumulator_check, get_srs},
    transition_chain,
    verification::verify,
    verifier_index::get_verifier_index,
};

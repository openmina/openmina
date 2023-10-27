#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::result_unit_err)]
// #![forbid(clippy::needless_pass_by_ref_mut)]

// Unused, we don't want to print on stdout
// /// Print logs on stdout with the prefix `[ledger]`
// macro_rules! log {
//     () => (elog!("[ledger]"));
//     ($($arg:tt)*) => ({
//         println!("[ledger] {}", format_args!($($arg)*))
//     })
// }

/// Print logs on stderr with the prefix `[ledger]`
macro_rules! elog {
    () => (elog!("[ledger]"));
    ($($arg:tt)*) => ({
        let _ = &format_args!($($arg)*);
        // eprintln!("[ledger] {}", format_args!($($arg)*));
    })
}

// We need a feature to tests both nodejs and browser
// https://github.com/rustwasm/wasm-bindgen/issues/2571
#[cfg(not(feature = "in_nodejs"))]
#[cfg(target_family = "wasm")]
#[cfg(test)]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

#[macro_use]
mod cache;

#[cfg(all(not(target_family = "wasm"), feature = "ocaml-interop"))]
mod ffi;

#[cfg(test)]
pub mod generators;

mod account;
mod address;
mod base;
// mod blocks;
mod database;
pub mod dummy;
mod hash;
pub mod mask;
pub mod ondisk;
mod port_ocaml;
mod poseidon;
pub mod proofs;
pub mod scan_state;
pub mod sparse_ledger;
pub mod staged_ledger;
mod tree;
mod tree_version;
mod util;
pub mod verifier;

pub use account::*;
pub use address::*;
pub use base::*;
// pub use blocks::*;
pub use database::*;
pub use hash::*;
pub use mask::*;
pub use poseidon::*;
pub use tree::*;
pub use tree_version::*;
pub use util::*;

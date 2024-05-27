#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::result_unit_err)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::needless_pub_self)]
#![allow(clippy::borrow_deref_ref, clippy::useless_conversion)]
#![allow(clippy::redundant_pattern_matching, clippy::map_flatten, clippy::let_and_return, clippy::module_inception)]
#![allow(clippy::to_string_trait_impl, clippy::bool_comparison, clippy::inherent_to_string, clippy::while_let_on_iterator)]
#![allow(clippy::manual_try_fold,clippy::into_iter_on_ref, clippy::map_identity, clippy::manual_range_patterns)]
#![allow(clippy::explicit_auto_deref, clippy::match_ref_pats, clippy::match_like_matches_macro)]
#![allow(clippy::get_first, clippy::blocks_in_conditions, clippy::comparison_chain, clippy::assign_op_pattern)]
#![allow(clippy::unnecessary_fallible_conversions, clippy::non_canonical_partial_ord_impl, clippy::redundant_guards)]
#![allow(clippy::needless_return, clippy::redundant_closure, clippy::redundant_static_lifetimes)]
#![allow(clippy::assigning_clones, clippy::unnecessary_cast, clippy::needless_lifetimes, clippy::new_without_default)]
#![allow(clippy::needless_borrows_for_generic_args, clippy::filter_map_identity, clippy::iter_cloned_collect, clippy::op_ref)]
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
pub mod zkapps;

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

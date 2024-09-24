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

#[cfg(any(test, feature = "fuzzing"))]
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
pub mod proofs;
pub mod scan_state;
pub mod sparse_ledger;
pub mod staged_ledger;
pub mod transaction_pool;
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
pub use tree::*;
pub use tree_version::*;
pub use util::*;

#[cfg(target_family = "wasm")]
mod wasm_tests {
    use super::*;

    use wasm_bindgen::prelude::*;

    // Called when the wasm module is instantiated
    #[wasm_bindgen(start)]
    fn main() -> Result<(), JsValue> {
        // Use `web_sys`'s global `window` function to get a handle on the global
        // window object.
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        // Manufacture the element we're gonna append
        let val = document.create_element("p")?;
        val.set_inner_html("Hello from Rust!");

        body.append_child(&val)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn call_reconstruct_staged_ledger(a: u32, b: u32) -> u32 {
        my_run();
        a + b
    }

    use std::collections::BTreeMap;

    use mina_hasher::Fp;
    use mina_p2p_messages::{binprot, list::List, v2};
    // use mina_tree::*;
    use scan_state::{
        pending_coinbase::PendingCoinbase, scan_state::ScanState,
        transaction_logic::local_state::LocalState,
    };
    use staged_ledger::staged_ledger::StagedLedger;
    use verifier::Verifier;

    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);

        // The `console.log` is quite polymorphic, so we can bind it with multiple
        // signatures. Note that we need to use `js_name` to ensure we always call
        // `log` in JS.
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        fn log_u32(a: u32);

        // Multiple arguments too!
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        fn log_many(a: &str, b: &str);
    }

    fn my_run() {
        log("Hello from Rust!!!!");

        #[allow(unused)]
        use binprot::{
            macros::{BinProtRead, BinProtWrite},
            BinProtRead, BinProtWrite,
        };

        #[derive(BinProtRead, BinProtWrite)]
        struct ReconstructContext {
            accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
            scan_state: v2::TransactionSnarkScanStateStableV2,
            pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
            staged_ledger_hash: v2::LedgerHash,
            states: List<v2::MinaStateProtocolStateValueStableV2>,
        }

        // let now = std::time::Instant::now();

        let file = include_bytes!("/tmp/failed_reconstruct_ctx.binprot");

        // // let Ok(file) = std::fs::read("/tmp/failed_reconstruct_ctx.binprot") else {
        // let Ok(file) = include_bytes!("/tmp/failed_reconstruct_ctx.binprot") else {
        //     eprintln!("no reconstruct context found");
        //     return;
        // };

        let ReconstructContext {
            accounts,
            scan_state,
            pending_coinbase,
            staged_ledger_hash,
            states,
        } = ReconstructContext::binprot_read(&mut file.as_slice()).unwrap();

        log("222");

        let states = states
            .iter()
            .map(|state| {
                (
                    state.try_hash().unwrap().to_field::<Fp>().unwrap(),
                    state.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        log("333");

        const LEDGER_DEPTH: usize = 35;
        let mut ledger = Mask::create(LEDGER_DEPTH);
        for account in &accounts {
            let account: Account = account.try_into().unwrap();
            let id = account.id();
            ledger.get_or_create_account(id, account).unwrap();
        }
        assert_eq!(ledger.num_accounts(), accounts.len());

        let instant_now = || {
            web_sys::window()
                .expect("should have a Window")
                .performance()
                .expect("should have a Performance")
                .now()
        };

        log("444");
        // eprintln!("time to parse and restore state: {:?}", now.elapsed());
        // let now = std::time::Instant::now();

        let now = instant_now();

        // for naccounts in [200] {
        //     // println!("{:?} accounts wasmer", naccounts);
        //     // let now = redux::Instant::now();
        //     let mut db = Database::<V2>::create(20);
        //     let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();
        //     for (index, mut account) in accounts.into_iter().enumerate() {
        //         account.token_id = TokenId::from(index as u64);
        //         let id = account.id();
        //         db.get_or_create_account(id, account).unwrap();
        //     }
        //     // println!("generate random accounts {:?}", now.elapsed());
        //     // let now = redux::Instant::now();
        //     assert_eq!(db.num_accounts(), naccounts as usize);
        //     db.merkle_root();
        //     // println!("compute merkle root {:?}", now.elapsed());
        // }

        let scan_state: ScanState = (&scan_state).try_into().unwrap();

        let now2 = instant_now();

        let ms = std::time::Duration::from_millis(now2 as u64 - now as u64);
        log(&format!("convert scan state: {:?}", ms));
        let now3 = instant_now();

        let staged_ledger_hash: Fp = staged_ledger_hash.0.to_field().unwrap();

        let ms = std::time::Duration::from_millis(now3 as u64 - now2 as u64);
        log(&format!("convert staged_ledger_hash: {:?}", ms));

        let now4 = instant_now();
        let pending_coinbase: PendingCoinbase = (&pending_coinbase).try_into().unwrap();

        let now5 = instant_now();
        let ms = std::time::Duration::from_millis(now5 as u64 - now4 as u64);
        log(&format!("convert pending_coinbase: {:?}", ms));

        let now6 = instant_now();

        // let root = ledger.merkle_root();
        let first_account: Account = (&accounts[0]).try_into().unwrap();
        let hash = first_account.hash();
        log(&format!("first_account_hash={:?}", hash));

        let now6 = instant_now();
        let root = ledger.merkle_root();
        log(&format!("root={:?}", root));
        // let mut staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
        //     (),
        //     openmina_core::constants::constraint_constants(),
        //     Verifier,
        //     scan_state,
        //     ledger,
        //     LocalState::empty(),
        //     staged_ledger_hash,
        //     pending_coinbase,
        //     |key| states.get(&key).cloned().unwrap(),
        // )
        // .unwrap();

        let now7 = instant_now();

        let ms = std::time::Duration::from_millis(now7 as u64 - now6 as u64);
        log(&format!("reconstruct: {:?}", ms));

        // eprintln!("time to reconstruct: {:?}", now.elapsed());
        // let now = std::time::Instant::now();
        // dbg!(staged_ledger.hash());
        // eprintln!("time to hash: {:?}", now.elapsed());

        // for naccounts in [1_000] {
        //     println!("{:?} accounts wasmer", naccounts);

        //     let now = redux::Instant::now();

        //     let mut db = Database::<V2>::create(20);

        //     let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

        //     for (index, mut account) in accounts.into_iter().enumerate() {
        //         account.token_id = TokenId::from(index as u64);
        //         let id = account.id();
        //         db.get_or_create_account(id, account).unwrap();
        //     }

        //     println!("generate random accounts {:?}", now.elapsed());
        //     let now = redux::Instant::now();

        //     assert_eq!(db.num_accounts(), naccounts as usize);

        //     db.merkle_root();

        //     println!("compute merkle root {:?}", now.elapsed());
        // }
    }
}

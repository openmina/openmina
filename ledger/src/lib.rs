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
use wasm_bindgen::prelude::*;


#[cfg(target_family = "wasm")]
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

    // #[wasm_bindgen]
    // extern "C" {

    // #[wasm_bindgen(js_namespace = performance, js_name = now)]
    // fn now() -> f64;

    // #[no_mangle]
    // #[used]
    // static performance: web_sys::Performance;
    // }
}

#[cfg(target_family = "wasm")]
mod prover {

    use std::{collections::HashMap, sync::Arc};

    use kimchi::circuits::gate::CircuitGate;
    use mina_curves::pasta::Fq;
    use mina_hasher::Fp;
    use proofs::{constants::{ProofConstants, StepBlockProof, WrapBlockProof, WrapTransactionProof}, field::FieldWitness, provers::{decode_constraints_data, BlockProver}, transaction::{make_prover_index, InternalVars, Prover, V}};

    use super::*;

    fn decode_gates_file<F: FieldWitness>(
        reader: impl std::io::Read,
    ) -> std::io::Result<Vec<CircuitGate<F>>> {
        #[serde_with::serde_as]
        #[derive(serde::Deserialize)]
        struct GatesFile<F: ark_ff::PrimeField> {
            public_input_size: usize,
            #[serde_as(as = "Vec<_>")]
            gates: Vec<CircuitGate<F>>,
        }
        let data: GatesFile<F> = serde_json::from_reader(reader)?;
        Ok(data.gates)
    }

    fn read_gates_file<F: FieldWitness>(
        gates_bytes: &[u8],
    ) -> std::io::Result<Vec<CircuitGate<F>>> {
        decode_gates_file(gates_bytes)
    }

    fn read_constraints_data<F: FieldWitness>(
        internal_vars_bytes: &[u8],
        rows_rev_bytes: &[u8],
    ) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
        decode_constraints_data(internal_vars_bytes, rows_rev_bytes)
    }

    fn make_gates<F: FieldWitness>(
        internal_vars_bytes: &[u8],
        gates_bytes: &[u8],
        rows_rev_bytes: &[u8],
    ) -> (
        HashMap<usize, (Vec<(F, V)>, Option<F>)>,
        Vec<Vec<Option<V>>>,
        Vec<CircuitGate<F>>,
    ) {
        let gates: Vec<CircuitGate<F>> = read_gates_file(&gates_bytes).unwrap();
        let (internal_vars_path, rows_rev_path) =
            read_constraints_data::<F>(&internal_vars_bytes, &rows_rev_bytes).unwrap();
        (internal_vars_path, rows_rev_path, gates)
    }

    fn get<C: ProofConstants, F: FieldWitness>(
        internal_vars_bytes: &[u8],
        gates_bytes: &[u8],
        rows_rev_bytes: &[u8],
    ) -> Arc<Prover<F>> {
        let (internal_vars, rows_rev, gates) = make_gates::<F>(internal_vars_bytes, gates_bytes, rows_rev_bytes);

        let index = make_prover_index::<C, _>(gates, None);
        Arc::new(Prover {
            internal_vars,
            rows_rev,
            index,
        })
    }

    pub fn get_block_prover() -> BlockProver {
        log(&format!("AAA"));
        let block_step_prover = get::<StepBlockProof, Fp>(
            include_bytes!("../3.0.1devnet/step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_internal_vars.bin"),
            include_bytes!("../3.0.1devnet/step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_gates.json"),
            include_bytes!("../3.0.1devnet/step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_rows_rev.bin"),
        );
        log(&format!("BBB"));
        let block_wrap_prover = get::<WrapBlockProof, Fq>(
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_internal_vars.bin"),
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_gates.json"),
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_rows_rev.bin"),
        );
        log(&format!("CCC"));
        let tx_wrap_prover = get::<WrapTransactionProof, Fq>(
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_internal_vars.bin"),
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_gates.json"),
            include_bytes!("../3.0.1devnet/wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_rows_rev.bin"),
        );

        BlockProver {
            block_step_prover,
            block_wrap_prover,
            tx_wrap_prover,
        }
    }
}

// Run with:
//
// export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128 -C link-arg=--max-memory=4294967296"
// rustup run nightly wasm-pack build --target web -- . -Z build-std=std,panic_abort && python3 -m http.server 8080
//
// To profile more than 5 seconds in chrome:
// https://stackoverflow.com/questions/54278305/when-does-start-profiling-and-reload-page-decide-to-stop-the-automatic-recordi

#[cfg(target_family = "wasm")]
mod wasm_tests {
    use super::*;

    use proofs::provers::BlockProver;
    use wasm_bindgen::prelude::*;

    use ::openmina_core::thread;
    use wasm_bindgen::JsValue;

    /// This method must be called to initialize rayon.
    /// This is an async function, and the verification code must be called only after `init_rayon` returned.
    /// This must not be called from the main thread.
    pub async fn init_rayon() -> Result<(), JsValue> {
        let num_cpus = thread::available_parallelism()
            .map_err(|err| format!("failed to get available parallelism: {err}"))?
            .get();

        thread::spawn(move || {
            rayon::ThreadPoolBuilder::new()
                .spawn_handler(|thread| {
                    thread::spawn(move || thread.run());
                    Ok(())
                })
                .num_threads(num_cpus.max(2) - 1)
                .build_global()
                .map_err(|e| format!("{:?}", e))
        })
            .join_async()
            .await
            .unwrap()?;

        Ok(())
    }

    fn run_prover_impl() {
        // https://github.com/rustwasm/wasm-bindgen/issues/1752
        let performance = js_sys::Reflect::get(&js_sys::global(), &"performance".into())
            .expect("failed to get performance from global object")
            .unchecked_into::<web_sys::Performance>();
        let instant_now = || {
            performance.now()
        };

        let now = instant_now();
        let BlockProver {
            block_step_prover,
            block_wrap_prover,
            tx_wrap_prover,
        } = prover::get_block_prover();
        let now2 = instant_now();
        let ms = std::time::Duration::from_millis(now2 as u64 - now as u64);
        log(&format!("get_block_prover: {:?}", ms));

        let data = include_bytes!("../3.0.1devnet/tests/block_input-2483246-0.bin");

        use crate::proofs::wrap::WrapProof;
        use crate::proofs::block::BlockParams;
        use crate::proofs::util::sha256_sum;
        use crate::proofs::generate_block_proof;

        let blockchain_input: v2::ProverExtendBlockchainInputStableV2 =
            read_binprot(&mut data.as_slice());

        let now = instant_now();
        let WrapProof { proof, .. } = generate_block_proof(
            BlockParams {
                input: &blockchain_input,
                block_step_prover: &block_step_prover,
                block_wrap_prover: &block_wrap_prover,
                tx_wrap_prover: &tx_wrap_prover,
                only_verify_constraints: false,
                expected_step_proof: None,
                ocaml_wrap_witness: None,
                // expected_step_proof: Some(
                //     "a82a10e5c276dd6dc251241dcbad005201034ffff5752516a179f317dfe385f5",
                // ),
                // ocaml_wrap_witness: Some(read_witnesses("block_fqs.txt").unwrap()),
            },
            // &mut witnesses,
        )
            .unwrap();
        let now2 = instant_now();
        let ms = std::time::Duration::from_millis(now2 as u64 - now as u64);
        log(&format!("prove block: {:?}", ms));
        let proof_json = serde_json::to_vec(&proof.proof).unwrap();
        let _sum = dbg!(sha256_sum(&proof_json));

        let now = instant_now();
        let WrapProof { proof, .. } = generate_block_proof(
            BlockParams {
                input: &blockchain_input,
                block_step_prover: &block_step_prover,
                block_wrap_prover: &block_wrap_prover,
                tx_wrap_prover: &tx_wrap_prover,
                only_verify_constraints: false,
                expected_step_proof: None,
                ocaml_wrap_witness: None,
                // expected_step_proof: Some(
                //     "a82a10e5c276dd6dc251241dcbad005201034ffff5752516a179f317dfe385f5",
                // ),
                // ocaml_wrap_witness: Some(read_witnesses("block_fqs.txt").unwrap()),
            },
            // &mut witnesses,
        )
            .unwrap();
        let now2 = instant_now();
        let ms = std::time::Duration::from_millis(now2 as u64 - now as u64);
        log(&format!("prove block: {:?}", ms));
        let proof_json = serde_json::to_vec(&proof.proof).unwrap();
        let _sum = dbg!(sha256_sum(&proof_json));
    }


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

    fn run_prover() {
        ::openmina_core::thread::main_thread_init();
        wasm_bindgen_futures::spawn_local(async {
            console_error_panic_hook::set_once();
            // tracing::initialize(tracing::Level::INFO);
            init_rayon().await.unwrap();
        });

        ::openmina_core::thread::spawn(|| {
            run_prover_impl();
        });
    }

    #[wasm_bindgen]
    pub fn call_reconstruct_staged_ledger(a: u32, b: u32) -> u32 {
        // Comment one or the other
        // run_prover();
        run_reconstruct();
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

    fn read_binprot<T, R>(mut r: R) -> T
    where
        T: binprot::BinProtRead,
        R: std::io::Read,
    {
        use std::io::Read;

        let mut len_buf = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut len_buf).unwrap();
        let len = u64::from_le_bytes(len_buf);

        let mut buf = Vec::with_capacity(len as usize);
        let mut r = r.take(len);
        r.read_to_end(&mut buf).unwrap();

        let mut read = buf.as_slice();
        T::binprot_read(&mut read).unwrap()
    }

    fn run_reconstruct() {
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

        let instant_now = || {
            web_sys::window()
                .expect("should have a Window")
                .performance()
                .expect("should have a Performance")
                .now()
        };

        log(&format!("ICI: {:?}", instant_now()));


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
        let now7 = instant_now();
        let ms = std::time::Duration::from_millis(now7 as u64 - now6 as u64);
        log(&format!("compute merkle_root: {:?}", ms));
        log(&format!("root_hash={:?}", root));

        let now8 = instant_now();

        let mut staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            openmina_core::constants::constraint_constants(),
            Verifier,
            scan_state,
            ledger,
            LocalState::empty(),
            staged_ledger_hash,
            pending_coinbase,
            |key| states.get(&key).cloned().unwrap(),
        )
        .unwrap();

        let now9 = instant_now();

        let ms = std::time::Duration::from_millis(now9 as u64 - now8 as u64);
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

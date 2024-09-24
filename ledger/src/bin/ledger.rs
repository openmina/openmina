// use wasm_bindgen::prelude::*;

// // Called when the wasm module is instantiated
// #[wasm_bindgen(start)]
// fn main() -> Result<(), JsValue> {
//     // Use `web_sys`'s global `window` function to get a handle on the global
//     // window object.
//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("should have a document on window");
//     let body = document.body().expect("document should have a body");

//     // Manufacture the element we're gonna append
//     let val = document.create_element("p")?;
//     val.set_inner_html("Hello from Rust!");

//     body.append_child(&val)?;

//     Ok(())
// }

// #[wasm_bindgen]
// pub fn add(a: u32, b: u32) -> u32 {
//     a + b
// }

// use std::collections::BTreeMap;

// use mina_hasher::Fp;
// use mina_p2p_messages::{binprot, list::List, v2};
// use mina_tree::*;
// use scan_state::transaction_logic::local_state::LocalState;
// use staged_ledger::staged_ledger::StagedLedger;
// use verifier::Verifier;

// fn main() {

//     #[allow(unused)]
//     use binprot::{
//         macros::{BinProtRead, BinProtWrite},
//         BinProtRead, BinProtWrite,
//     };

//     #[derive(BinProtRead, BinProtWrite)]
//     struct ReconstructContext {
//         accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
//         scan_state: v2::TransactionSnarkScanStateStableV2,
//         pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
//         staged_ledger_hash: v2::LedgerHash,
//         states: List<v2::MinaStateProtocolStateValueStableV2>,
//     }

//     let now = std::time::Instant::now();

//     let Ok(file) = std::fs::read("/tmp/failed_reconstruct_ctx.binprot") else {
//         eprintln!("no reconstruct context found");
//         return;
//     };

//     let ReconstructContext {
//         accounts,
//         scan_state,
//         pending_coinbase,
//         staged_ledger_hash,
//         states,
//     } = ReconstructContext::binprot_read(&mut file.as_slice()).unwrap();

//     let states = states
//         .iter()
//         .map(|state| {
//             (
//                 state.try_hash().unwrap().to_field::<Fp>().unwrap(),
//                 state.clone(),
//             )
//         })
//         .collect::<BTreeMap<_, _>>();

//     const LEDGER_DEPTH: usize = 35;
//     let mut ledger = Mask::create(LEDGER_DEPTH);
//     for account in &accounts {
//         let account: Account = account.try_into().unwrap();
//         let id = account.id();
//         ledger.get_or_create_account(id, account).unwrap();
//     }
//     assert_eq!(ledger.num_accounts(), accounts.len());

//     eprintln!("time to parse and restore state: {:?}", now.elapsed());
//     let now = std::time::Instant::now();

//     let mut staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
//         (),
//         openmina_core::constants::constraint_constants(),
//         Verifier,
//         (&scan_state).try_into().unwrap(),
//         ledger,
//         LocalState::empty(),
//         staged_ledger_hash.0.to_field().unwrap(),
//         (&pending_coinbase).try_into().unwrap(),
//         |key| states.get(&key).cloned().unwrap(),
//     )
//         .unwrap();

//     eprintln!("time to reconstruct: {:?}", now.elapsed());
//     let now = std::time::Instant::now();
//     dbg!(staged_ledger.hash());
//     eprintln!("time to hash: {:?}", now.elapsed());

//     // for naccounts in [1_000] {
//     //     println!("{:?} accounts wasmer", naccounts);

//     //     let now = redux::Instant::now();

//     //     let mut db = Database::<V2>::create(20);

//     //     let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

//     //     for (index, mut account) in accounts.into_iter().enumerate() {
//     //         account.token_id = TokenId::from(index as u64);
//     //         let id = account.id();
//     //         db.get_or_create_account(id, account).unwrap();
//     //     }

//     //     println!("generate random accounts {:?}", now.elapsed());
//     //     let now = redux::Instant::now();

//     //     assert_eq!(db.num_accounts(), naccounts as usize);

//     //     db.merkle_root();

//     //     println!("compute merkle root {:?}", now.elapsed());
//     // }
// }

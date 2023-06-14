use std::{collections::BTreeMap, fs::File};

use crate::{
    scan_state::{
        currency::{Amount, Fee},
        scan_state::ConstraintConstants,
        transaction_logic::local_state::LocalState,
    },
    staged_ledger::staged_ledger::StagedLedger,
    verifier::Verifier,
    Account, BaseLedger, Database, Mask,
};
use binprot::BinProtRead;
use mina_p2p_messages::{
    hash::MinaHash, rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response, v2,
};

const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: Amount::from_u64(720000000000),
    supercharged_coinbase_factor: 2,
    account_creation_fee: Fee::from_u64(1000000000),
    fork: None,
};

#[test]
fn staged_ledger_hash() {
    let mut snarked_ledger_file = File::open("target/snarked_ledger").unwrap();
    let mut snarked_ledger = Mask::new_root(Database::create(35));
    while let Ok(account) = Account::binprot_read(&mut snarked_ledger_file) {
        let account_id = account.id();
        snarked_ledger
            .get_or_create_account(account_id, account)
            .unwrap();
    }

    let mut staged_ledger_file = File::open("target/staged_ledger").unwrap();

    let info = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response::binprot_read(
        &mut staged_ledger_file,
    )
    .unwrap();

    println!("Prepare snarked ledger");

    let (scan_state, expected_ledger_hash, pending_coinbase, states) = info.unwrap();
    let states = states
        .into_iter()
        .map(|state| (state.hash(), state))
        .collect::<BTreeMap<_, _>>();

    println!("Load staged ledger info");

    let mut staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
        (),
        &CONSTRAINT_CONSTANTS,
        Verifier,
        (&scan_state).into(),
        snarked_ledger,
        LocalState::empty(),
        expected_ledger_hash.clone().into(),
        (&pending_coinbase).into(),
        |key| states.get(&key).cloned().unwrap(),
    )
    .unwrap();

    println!("Prepare staged ledger");

    let hash = v2::MinaBaseStagedLedgerHashStableV1::from(&staged_ledger.hash());
    let reference = r#"{"non_snark":{"ledger_hash":"jx1NihaovYTCrj2R3hMkM2yZ2FYXyENfH5VGrPnMGTLtUizot6W","aux_hash":"W5wyeVGcLw1JBxiyMnqDQ46WCrQBQebxMKfRQWRFzHxD7Zn7Dz","pending_coinbase_aux":"WVjrHtNPbda9FeMD6Lx7aU9Y58KiW3hmBgE488dfiSWSHUSiLD"},"pending_coinbase_hash":"2n1HWMiBfccZf52vcNyVsGVu6eYthCYez52mK72SWJtzdmWQHSq9"}"#;
    assert_eq!(reference, serde_json::to_string(&hash).unwrap());
}

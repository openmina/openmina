use crate::transaction_fuzzer::{
    generator::{Generator, GeneratorRange32, GeneratorRange64},
    mutator::Mutator,
    {deserialize, serialize},
};
use ark_ff::fields::arithmetic::InvalidBigInt;
use ark_ff::Zero;
use ledger::scan_state::transaction_logic::transaction_applied::{
    signed_command_applied, CommandApplied, TransactionApplied, Varying,
};
use ledger::scan_state::transaction_logic::{
    apply_transactions, Transaction, TransactionStatus, UserCommand,
};
use ledger::sparse_ledger::LedgerIntf;
use ledger::staged_ledger::staged_ledger::StagedLedger;
use ledger::{dummy, Account, AccountId, Database, Mask, Timing, TokenId};
use ledger::{
    scan_state::currency::{Amount, Fee, Length, Magnitude, Nonce, Signed, Slot},
    transaction_pool::TransactionPool,
};
use ledger::{
    scan_state::transaction_logic::protocol_state::{
        protocol_state_view, EpochData, EpochLedger, ProtocolStateView,
    },
    transaction_pool,
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::binprot::SmallString1k;
use mina_p2p_messages::{
    bigint, binprot,
    v2::{
        MinaTransactionLogicTransactionAppliedStableV2,
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    },
};
use mina_signer::{CompressedPubKey, Keypair};
use node::DEVNET_CONFIG;
use openmina_core::{consensus::ConsensusConstants, constants::ConstraintConstants, NetworkConfig};
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use ring_buffer::RingBuffer;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::{fs, str::FromStr};

// Taken from ocaml_tests
/// Same values when we run `dune runtest src/lib/staged_ledger -f`
#[coverage(off)]
fn dummy_state_view(global_slot_since_genesis: Option<Slot>) -> ProtocolStateView {
    // TODO: Use OCaml implementation, not hardcoded value
    let f = #[coverage(off)]
    |s: &str| Fp::from_str(s).unwrap();

    ProtocolStateView {
        snarked_ledger_hash: f(
            "19095410909873291354237217869735884756874834695933531743203428046904386166496",
        ),
        blockchain_length: Length::from_u32(1),
        min_window_density: Length::from_u32(77),
        total_currency: Amount::from_u64(10016100000000000),
        global_slot_since_genesis: global_slot_since_genesis.unwrap_or_else(Slot::zero),
        staking_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: f(
                    "19095410909873291354237217869735884756874834695933531743203428046904386166496",
                ),
                total_currency: Amount::from_u64(10016100000000000),
            },
            seed: Fp::zero(),
            start_checkpoint: Fp::zero(),
            lock_checkpoint: Fp::zero(),
            epoch_length: Length::from_u32(1),
        },
        next_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: f(
                    "19095410909873291354237217869735884756874834695933531743203428046904386166496",
                ),
                total_currency: Amount::from_u64(10016100000000000),
            },
            seed: f(
                "18512313064034685696641580142878809378857342939026666126913761777372978255172",
            ),
            start_checkpoint: Fp::zero(),
            lock_checkpoint: f(
                "9196091926153144288494889289330016873963015481670968646275122329689722912273",
            ),
            epoch_length: Length::from_u32(2),
        },
    }
}

#[coverage(off)]
pub fn dummy_state_and_view(
    global_slot: Option<Slot>,
) -> Result<
    (
        mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2,
        ProtocolStateView,
    ),
    InvalidBigInt,
> {
    let mut state = dummy::for_tests::dummy_protocol_state();

    if let Some(global_slot) = global_slot {
        let new_global_slot = global_slot;

        let global_slot_since_genesis = {
            let since_genesis = &state.body.consensus_state.global_slot_since_genesis;
            let curr = &state
                .body
                .consensus_state
                .curr_global_slot_since_hard_fork
                .slot_number;

            let since_genesis = Slot::from_u32(since_genesis.as_u32());
            let curr = Slot::from_u32(curr.as_u32());

            (since_genesis.checked_sub(&curr).unwrap())
                .checked_add(&new_global_slot)
                .unwrap()
        };

        let cs = &mut state.body.consensus_state;
        cs.curr_global_slot_since_hard_fork.slot_number = (&new_global_slot).into();
        cs.global_slot_since_genesis = (&global_slot_since_genesis).into();
    };

    let view = protocol_state_view(&state)?;

    Ok((state, view))
}

pub enum PermissionModel {
    Any,        // Allow any (random) combination of permissions
    Empty,      // Permissions are always set to None
    Initial,    // Permissions are always set to "user_default" set (signature only).
    Default,    // "default" permissions as set by SnarkyJS when deploying a zkApp.
    TokenOwner, // permission set usually set in Token owner zkApps
}

impl Clone for PermissionModel {
    #[coverage(off)]
    fn clone(&self) -> Self {
        match self {
            PermissionModel::Any => PermissionModel::Any,
            PermissionModel::Empty => PermissionModel::Empty,
            PermissionModel::Initial => PermissionModel::Initial,
            PermissionModel::Default => PermissionModel::Default,
            PermissionModel::TokenOwner => PermissionModel::TokenOwner,
        }
    }
}

#[derive(Debug)]
pub struct ApplyTxResult {
    root_hash: Fp,
    pub apply_result: Vec<TransactionApplied>,
    error: String,
}

impl binprot::BinProtRead for ApplyTxResult {
    #[coverage(off)]
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let root_hash: Fp = bigint::BigInt::binprot_read(r)?
            .try_into()
            .map_err(|x| binprot::Error::CustomError(Box::new(x)))?;
        // Start of Selection
        let apply_result = Vec::<MinaTransactionLogicTransactionAppliedStableV2>::binprot_read(r)?
            .into_iter()
            .map(
                #[coverage(off)]
                |MinaTransactionLogicTransactionAppliedStableV2 {
                     previous_hash,
                     varying,
                 }| {
                    let previous_hash = previous_hash
                        .0
                        .to_field()
                        .map_err(|x| binprot::Error::CustomError(Box::new(x)))?;

                    let varying = (&varying)
                        .try_into()
                        .map_err(|x| binprot::Error::CustomError(Box::new(x)))?;

                    Ok::<TransactionApplied, binprot::Error>(TransactionApplied {
                        previous_hash,
                        varying,
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        let error: String = SmallString1k::binprot_read(r)?.0;

        Ok(ApplyTxResult {
            root_hash,
            apply_result,
            error,
        })
    }
}

pub enum LedgerKind {
    Mask(Mask),
    Staged(StagedLedger, Mask),
}

impl Clone for LedgerKind {
    #[coverage(off)]
    fn clone(&self) -> Self {
        match self {
            Self::Mask(ledger) => Self::Mask(ledger.copy()),
            Self::Staged(ledger, snarked_ledger) => {
                Self::Staged(ledger.clone(), snarked_ledger.clone())
            }
        }
    }
}

pub struct FuzzerState {
    pub ledger: LedgerKind,
    pub potential_senders: Vec<(Keypair, PermissionModel)>,
    pub potential_new_accounts: Vec<(Keypair, PermissionModel)>,
    pub cache_pool: RingBuffer<UserCommand>,
    pub cache_apply: RingBuffer<UserCommand>,
    pub cache_curve_point_fp: Option<(Fp, Fp)>,
    pub cache_curve_point_fq: Option<(Fq, Fq)>,
}

impl Clone for FuzzerState {
    #[coverage(off)]
    fn clone(&self) -> Self {
        Self {
            ledger: self.ledger.clone(),
            potential_senders: self.potential_senders.clone(),
            potential_new_accounts: self.potential_new_accounts.clone(),
            cache_pool: self.cache_pool.clone(),
            cache_apply: self.cache_apply.clone(),
            cache_curve_point_fp: self.cache_curve_point_fp,
            cache_curve_point_fq: self.cache_curve_point_fq,
        }
    }
}

pub struct GeneratorCtx {
    pub rng: SmallRng,
    pub max_account_balance: u64,
    pub minimum_fee: u64,
    pub excess_fee: Signed<Amount>,
    pub token_id: TokenId,
    pub tx_proof: Option<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>,
    pub nonces: HashMap<String, Nonce>, // TODO: implement hash trait for CompressedPubKey
    /// Attempt to produce a valid zkapp
    pub attempt_valid_zkapp: bool,
}

pub struct FuzzerCtx {
    pub constraint_constants: ConstraintConstants,
    pub txn_state_view: ProtocolStateView,
    pub pool: TransactionPool,
    pub fuzzcases_path: String,
    pub gen: GeneratorCtx,
    pub state: FuzzerState,
    pub snapshots: RingBuffer<FuzzerState>,
}

impl FuzzerCtx {
    #[coverage(off)]
    pub fn get_ledger_inner(&self) -> &Mask {
        match &self.state.ledger {
            LedgerKind::Mask(ledger) => ledger,
            LedgerKind::Staged(ledger, _) => ledger.ledger_ref(),
        }
    }

    #[coverage(off)]
    fn get_ledger_inner_mut(&mut self) -> &mut Mask {
        match &mut self.state.ledger {
            LedgerKind::Mask(ledger) => ledger,
            LedgerKind::Staged(ledger, _) => ledger.ledger_mut(),
        }
    }

    #[coverage(off)]
    fn get_snarked_ledger_inner_mut(&mut self) -> Option<&mut Mask> {
        match &mut self.state.ledger {
            LedgerKind::Mask(_) => None,
            LedgerKind::Staged(_, snarked_ledger) => Some(snarked_ledger),
        }
    }

    #[coverage(off)]
    pub fn get_snarked_ledger(&mut self) -> &mut Mask {
        match &mut self.state.ledger {
            LedgerKind::Staged(_, ledger) => ledger,
            _ => panic!(),
        }
    }

    #[coverage(off)]
    pub fn create_inital_accounts(&mut self, n: usize) {
        for _ in 0..n {
            loop {
                let keypair: Keypair = self.gen();

                if !self.state.potential_senders.iter().any(
                    #[coverage(off)]
                    |(kp, _)| kp.public == keypair.public,
                ) {
                    let pk_compressed = keypair.public.into_compressed();
                    let account_id = AccountId::new(pk_compressed, TokenId::default());
                    let mut account = Account::initialize(&account_id);

                    account.balance = GeneratorRange64::gen_range(
                        self,
                        1_000_000_000_000..=self.gen.max_account_balance,
                    );
                    account.nonce = GeneratorRange32::gen_range(self, 0..=u32::MAX);
                    account.timing = Timing::Untimed;

                    let permission_model = PermissionModel::Any; //self.gen();
                    self.state
                        .potential_senders
                        .push((keypair, permission_model));

                    if let Some(snarked_ledger) = self.get_snarked_ledger_inner_mut() {
                        snarked_ledger
                            .create_new_account(account_id.clone(), account.clone())
                            .unwrap();
                    };

                    self.get_ledger_inner_mut()
                        .create_new_account(account_id, account)
                        .unwrap();

                    break;
                }
            }
        }
    }

    #[coverage(off)]
    pub fn get_account(&self, pkey: &CompressedPubKey) -> Option<Account> {
        self.get_account_by_id(&AccountId::new(pkey.clone(), TokenId::default()))
    }

    #[coverage(off)]
    pub fn get_account_by_id(&self, account_id: &AccountId) -> Option<Account> {
        let account_location = LedgerIntf::location_of_account(self.get_ledger_inner(), account_id);

        account_location.map(
            #[coverage(off)]
            |location| *(LedgerIntf::get(self.get_ledger_inner(), &location).unwrap()).clone(),
        )
    }

    #[coverage(off)]
    pub fn find_sender(&self, pkey: &CompressedPubKey) -> Option<&(Keypair, PermissionModel)> {
        self.state.potential_senders.iter().find(
            #[coverage(off)]
            |(kp, _)| kp.public.into_compressed() == *pkey,
        )
    }

    #[coverage(off)]
    pub fn find_permissions(&self, pkey: &CompressedPubKey) -> Option<&PermissionModel> {
        self.find_sender(pkey).map(
            #[coverage(off)]
            |(_, pm)| pm,
        )
    }

    #[coverage(off)]
    pub fn find_keypair(&self, pkey: &CompressedPubKey) -> Option<&Keypair> {
        self.find_sender(pkey).map(
            #[coverage(off)]
            |(kp, _)| kp,
        )
    }

    #[coverage(off)]
    pub fn random_keypair(&mut self) -> Keypair {
        self.state
            .potential_senders
            .choose(&mut self.gen.rng)
            .unwrap()
            .0
            .clone()
    }

    #[coverage(off)]
    pub fn random_ntransactions(&mut self) -> usize {
        self.gen.rng.gen_range(0..400)
    }

    #[coverage(off)]
    pub fn random_snark_worker_fee(&mut self) -> Fee {
        let fee = self.gen.rng.gen_range(0..10_000_000);
        Fee::from_u64(fee)
    }

    #[coverage(off)]
    pub fn random_user_command(&mut self) -> UserCommand {
        if self.gen.rng.gen_bool(0.9) {
            if !self.state.cache_apply.is_empty() {
                // Pick transaction from the applied tx cache and mutate it
                let index = self.gen.rng.gen_range(0..self.state.cache_apply.len());

                if let Some(mut transaction) = self.state.cache_apply.get_relative(index).cloned() {
                    self.mutate(&mut transaction);
                    return transaction;
                }
            }

            // If we can't find a tx in the applied cache, try one from the pool cache
            if self.gen.rng.gen_bool(0.5) && !self.state.cache_pool.is_empty() {
                let index = self.gen.rng.gen_range(0..self.state.cache_pool.len());

                if let Some(mut transaction) = self.state.cache_pool.get_relative(index).cloned() {
                    self.mutate(&mut transaction);
                    return transaction;
                }
            }
        }

        // Generate random transaction
        self.gen()
    }

    #[coverage(off)]
    pub fn random_tx_proof(
        &mut self,
    ) -> TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 {
        let mut proof = self
            .gen
            .tx_proof
            .clone()
            .expect("valid tx proof not set for FuzzerCtx");
        self.mutate(&mut proof);
        proof
    }

    #[coverage(off)]
    pub fn take_snapshot(&mut self) {
        println!("Taking snapshot...");
        self.snapshots.push_back(self.state.clone());
    }

    #[coverage(off)]
    pub fn restore_snapshot(&mut self) {
        if !self.snapshots.is_empty() {
            // Pick random snapshot
            let index = self.gen.rng.gen_range(0..self.snapshots.len());

            if let Some(state) = self.snapshots.get_relative(index).cloned() {
                println!("Restoring snapshot {}...", index);
                self.state = state;
            }
        }
    }

    #[coverage(off)]
    pub fn save_fuzzcase(&self, user_command: &UserCommand, filename: &str) {
        let filename = self.fuzzcases_path.clone() + filename + ".fuzzcase";

        println!("Saving fuzzcase: {}", filename);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(filename)
            .unwrap();

        serialize(
            &(self.get_ledger_accounts(), user_command.clone()),
            &mut file,
        );
    }

    #[coverage(off)]
    pub fn load_fuzzcase(&mut self, file_path: &String) -> UserCommand {
        println!("Loading fuzzcase: {}", file_path);
        let bytes = fs::read(file_path).unwrap();
        let (accounts, user_command): (Vec<Account>, UserCommand) =
            deserialize(&mut bytes.as_slice());

        let depth = self.constraint_constants.ledger_depth as usize;
        let root = Mask::new_root(Database::create(depth.try_into().unwrap()));

        *self.get_ledger_inner_mut() = root.make_child();

        for account in accounts {
            self.get_ledger_inner_mut()
                .create_new_account(account.id(), account)
                .unwrap();
        }

        user_command
    }

    #[coverage(off)]
    pub fn diagnostic(&self, applied: &impl Debug, applied_ocaml: &impl Debug) -> String {
        use text_diff::{diff, Difference};

        let orig = format!("{:#?}", applied);
        let edit = format!("{:#?}", applied_ocaml);
        let split = " ";
        let (_, changeset) = diff(orig.as_str(), edit.as_str(), split);

        let mut ret = String::new();

        for seq in changeset {
            match seq {
                Difference::Same(ref x) => {
                    ret.push_str(x);
                    ret.push_str(split);
                }
                Difference::Add(ref x) => {
                    ret.push_str("\x1B[92m");
                    ret.push_str(x);
                    ret.push_str("\x1B[0m");
                    ret.push_str(split);
                }
                Difference::Rem(ref x) => {
                    ret.push_str("\x1B[91m");
                    ret.push_str(x);
                    ret.push_str("\x1B[0m");
                    ret.push_str(split);
                }
            }
        }

        ret
    }

    #[coverage(off)]
    pub fn pool_verify(
        &self,
        user_command: &UserCommand,
        ocaml_pool_verify_result: &Result<Vec<UserCommand>, SmallString1k>,
    ) -> bool {
        let diff = transaction_pool::diff::Diff {
            list: vec![user_command.clone()],
        };
        let account_ids = user_command.accounts_referenced();
        let accounts = account_ids
            .iter()
            .filter_map(
                #[coverage(off)]
                |account_id| {
                    self.get_account_by_id(account_id).and_then(
                        #[coverage(off)]
                        |account| Some((account_id.clone(), account)),
                    )
                },
            )
            .collect::<BTreeMap<_, _>>();

        let rust_pool_result = self.pool.prevalidate(diff);
        let mismatch;

        if let Ok(diff) = rust_pool_result {
            let convert_diff_result = self.pool.convert_diff_to_verifiable(diff, &accounts);

            if let Ok(commands) = convert_diff_result {
                let verify_result = &ledger::verifier::Verifier.verify_commands(commands, None)[0];
                let ocaml_pool_verify_result = ocaml_pool_verify_result.clone().map(
                    #[coverage(off)]
                    |commands| commands[0].clone(),
                );

                *ledger::GLOBAL_SKIP_PARTIAL_EQ.write().unwrap() = true;
                mismatch = ocaml_pool_verify_result.is_ok()
                    && (verify_result.is_err()
                        || verify_result.as_ref().unwrap().forget_check()
                            != ocaml_pool_verify_result.clone().unwrap());

                if mismatch {
                    println!(
                        "verify_commands: Mismatch between Rust and OCaml pool_verify_result\n{}",
                        self.diagnostic(&verify_result, &ocaml_pool_verify_result)
                    );
                }
            } else {
                mismatch = ocaml_pool_verify_result.is_ok();

                if mismatch {
                    println!(
                            "convert_diff_to_verifiable: Mismatch between Rust and OCaml pool_verify_result\n{}",
                            self.diagnostic(&convert_diff_result, &ocaml_pool_verify_result)
                            );
                }
            }
        } else {
            mismatch = ocaml_pool_verify_result.is_ok();

            if mismatch {
                println!(
                    "prevalidate: Mismatch between Rust and OCaml pool_verify_result\n{}",
                    self.diagnostic(&rust_pool_result, &ocaml_pool_verify_result)
                );
            }
        }

        return mismatch;
    }

    #[coverage(off)]
    pub fn apply_transaction(
        &mut self,
        ledger: &mut Mask,
        user_command: &UserCommand,
        expected_apply_result: &ApplyTxResult,
    ) -> Result<(), String> {
        self.gen.nonces.clear();

        // If we called apply_transaction it means we passed the tx pool check, so add tx to the cache
        if let UserCommand::ZkAppCommand(command) = user_command {
            if !command.account_updates.is_empty() {
                //println!("Storing in pool cache {:?}", tx);
                self.state.cache_pool.push_back(user_command.clone());
            }
        }

        //println!("tx: {:?}\n", tx);
        let tx = Transaction::Command(user_command.clone());

        *ledger::GLOBAL_SKIP_PARTIAL_EQ.write().unwrap() = false;

        let applied = apply_transactions(
            &self.constraint_constants,
            self.txn_state_view.global_slot_since_genesis,
            &self.txn_state_view,
            ledger,
            &[tx.clone()],
        );

        *ledger::GLOBAL_SKIP_PARTIAL_EQ.write().unwrap() = true;

        match applied {
            Ok(applied) => {
                // For now we work with one transaction at a time
                let applied = &applied[0];

                if expected_apply_result.apply_result.len() != 1 {
                    return Err(format!(
                        "Apply failed in OCaml (error: {}) but it didn't in Rust: {:?}",
                        expected_apply_result.error, applied
                    ));
                } else if applied != &expected_apply_result.apply_result[0] {
                    return Err(format!(
                        "Apply result mismatch between OCaml and Rust\n{}\n",
                        self.diagnostic(applied, &expected_apply_result.apply_result[0])
                    ));
                }

                //println!("RUST: {:?}", applied);
                //println!("OCAML: {:?}", expected_apply_result);

                // Save applied transactions in the cache for later use (mutation)
                if *applied.transaction_status() == TransactionStatus::Applied {
                    if let UserCommand::ZkAppCommand(command) = user_command {
                        if !command.account_updates.is_empty() {
                            //println!("Storing in apply cache {:?}", tx);
                            self.state.cache_apply.push_back(user_command.clone());
                        }
                    }
                } else {
                    //println!("{:?}", applied.transaction_status());
                }

                // Add new accounts created by the transaction to the potential senders list
                let new_accounts = match &applied.varying {
                    Varying::Command(command) => match command {
                        CommandApplied::SignedCommand(cmd) => match &cmd.body {
                            signed_command_applied::Body::Payments { new_accounts } => {
                                Some(new_accounts)
                            }
                            _ => None,
                        },
                        CommandApplied::ZkappCommand(cmd) => Some(&cmd.new_accounts),
                    },
                    _ => unimplemented!(),
                };

                if let Some(new_accounts) = new_accounts {
                    let new_accounts = self.state.potential_new_accounts.iter().filter(
                        #[coverage(off)]
                        |(kp, _)| {
                            new_accounts.iter().any(
                                #[coverage(off)]
                                |acc| acc.public_key == kp.public.into_compressed(),
                            )
                        },
                    );

                    for acc in new_accounts {
                        if !self.state.potential_senders.iter().any(
                            #[coverage(off)]
                            |(kp, _)| kp.public == acc.0.public,
                        ) {
                            self.state.potential_senders.push(acc.clone())
                        }
                    }

                    self.state.potential_new_accounts.clear();
                }
            }
            Err(error_string) => {
                // Currently disabled until invariants are fixed
                if error_string.starts_with("Invariant violation") {
                    return Err(error_string);
                }

                if expected_apply_result.apply_result.len() == 1 {
                    return Err(format!(
                        "Apply failed in Rust (error: {}) but it didn't in OCaml: {:?}",
                        error_string, &expected_apply_result.apply_result[0]
                    ));
                } else {
                    //println!("ERROR RUST: {error_string}");
                    //println!("ERROR OCAML: {:?}", expected_apply_result);
                }
            }
        }

        let rust_ledger_root_hash = LedgerIntf::merkle_root(ledger);

        if expected_apply_result.root_hash != rust_ledger_root_hash {
            Err(format!(
                "Ledger hash mismatch: {:?} != {:?} (expected)",
                rust_ledger_root_hash, expected_apply_result.root_hash
            ))
        } else {
            ledger.commit();
            Ok(())
        }
    }

    #[coverage(off)]
    pub fn get_ledger_root(&mut self) -> Fp {
        LedgerIntf::merkle_root(self.get_ledger_inner_mut())
    }

    #[coverage(off)]
    pub fn get_ledger_accounts(&self) -> Vec<Account> {
        let locations = self.get_ledger_inner().account_locations();
        locations
            .iter()
            .map(
                #[coverage(off)]
                |x| *(LedgerIntf::get(self.get_ledger_inner(), x).unwrap()),
            )
            .collect()
    }
}

pub struct FuzzerCtxBuilder {
    constraint_constants: Option<ConstraintConstants>,
    txn_state_view: Option<ProtocolStateView>,
    pool: Option<TransactionPool>,
    fuzzcases_path: Option<String>,
    seed: u64,
    minimum_fee: u64,
    max_account_balance: u64,
    initial_accounts: usize,
    cache_size: usize,
    snapshots_size: usize,
    is_staged_ledger: bool,
}

impl Default for FuzzerCtxBuilder {
    #[coverage(off)]
    fn default() -> Self {
        Self {
            constraint_constants: None,
            txn_state_view: None,
            pool: None,
            fuzzcases_path: None,
            seed: 0,
            minimum_fee: 1_000_000,
            max_account_balance: 1_000_000_000_000_000,
            initial_accounts: 10,
            cache_size: 4096,
            snapshots_size: 128,
            is_staged_ledger: false,
        }
    }
}

impl FuzzerCtxBuilder {
    #[coverage(off)]
    pub fn new() -> Self {
        Self::default()
    }

    #[coverage(off)]
    pub fn constants(&mut self, constraint_constants: ConstraintConstants) -> &mut Self {
        self.constraint_constants = Some(constraint_constants);
        self
    }

    #[coverage(off)]
    pub fn state_view(&mut self, txn_state_view: ProtocolStateView) -> &mut Self {
        self.txn_state_view = Some(txn_state_view);
        self
    }

    pub fn transaction_pool(&mut self, pool: TransactionPool) -> &mut Self {
        self.pool = Some(pool);
        self
    }

    #[coverage(off)]
    pub fn fuzzcases_path(&mut self, fuzzcases_path: String) -> &mut Self {
        self.fuzzcases_path = Some(fuzzcases_path);
        self
    }

    #[coverage(off)]
    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.seed = seed;
        self
    }

    #[coverage(off)]
    pub fn minimum_fee(&mut self, minimum_fee: u64) -> &mut Self {
        self.minimum_fee = minimum_fee;
        self
    }

    #[coverage(off)]
    pub fn initial_accounts(&mut self, initial_accounts: usize) -> &mut Self {
        self.initial_accounts = initial_accounts;
        self
    }

    #[coverage(off)]
    pub fn cache_size(&mut self, cache_size: usize) -> &mut Self {
        assert!(cache_size != 0 && cache_size.is_power_of_two());
        self.cache_size = cache_size;
        self
    }

    #[coverage(off)]
    pub fn snapshots_size(&mut self, snapshots_size: usize) -> &mut Self {
        assert!(snapshots_size != 0 && snapshots_size.is_power_of_two());
        self.snapshots_size = snapshots_size;
        self
    }

    #[coverage(off)]
    pub fn is_staged_ledger(&mut self, is_staged_ledger: bool) -> &mut Self {
        self.is_staged_ledger = is_staged_ledger;
        self
    }

    #[coverage(off)]
    pub fn build(&mut self) -> FuzzerCtx {
        let mut constraint_constants = self
            .constraint_constants
            .clone()
            .unwrap_or(NetworkConfig::global().constraint_constants.clone());

        // HACK (binprot breaks in the OCaml side)
        constraint_constants.fork = None;

        let depth = constraint_constants.ledger_depth as usize;
        let root = Mask::new_root(Database::create(depth.try_into().unwrap()));
        let txn_state_view = self
            .txn_state_view
            .clone()
            .unwrap_or(dummy_state_view(None));

        let protocol_constants = DEVNET_CONFIG
            .protocol_constants()
            .expect("wrong protocol constants");

        let default_pool = TransactionPool::new(
            ledger::transaction_pool::Config {
                trust_system: (),
                pool_max_size: 3000,
                slot_tx_end: None,
            },
            &ConsensusConstants::create(&constraint_constants, &protocol_constants),
        );

        let pool = self.pool.clone().unwrap_or(default_pool);
        let fuzzcases_path = self.fuzzcases_path.clone().unwrap_or("./".to_string());

        let ledger = match self.is_staged_ledger {
            true => {
                let snarked_ledger_mask = root.make_child().fuzzing_to_root();
                LedgerKind::Staged(
                    StagedLedger::create_exn(constraint_constants.clone(), root.make_child())
                        .unwrap(),
                    snarked_ledger_mask,
                )
            }
            false => LedgerKind::Mask(root.make_child()),
        };

        let mut ctx = FuzzerCtx {
            constraint_constants,
            txn_state_view,
            pool,
            fuzzcases_path,
            gen: GeneratorCtx {
                rng: SmallRng::seed_from_u64(self.seed),
                minimum_fee: self.minimum_fee,
                excess_fee: Signed::<Amount>::zero(),
                token_id: TokenId::default(),
                tx_proof: None,
                max_account_balance: self.max_account_balance,
                nonces: HashMap::new(),
                attempt_valid_zkapp: true,
            },
            state: FuzzerState {
                ledger,
                potential_senders: Vec::new(),
                potential_new_accounts: Vec::new(),
                cache_pool: RingBuffer::with_capacity(self.cache_size),
                cache_apply: RingBuffer::with_capacity(self.cache_size),
                cache_curve_point_fp: None,
                cache_curve_point_fq: None,
            },
            snapshots: RingBuffer::with_capacity(self.snapshots_size),
        };

        ctx.create_inital_accounts(self.initial_accounts);
        ctx
    }
}

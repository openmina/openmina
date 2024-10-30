use crate::transaction_fuzzer::{
    generator::{Generator, GeneratorRange32, GeneratorRange64},
    mutator::Mutator,
    {deserialize, serialize},
};
use ark_ff::fields::arithmetic::InvalidBigInt;
use ark_ff::Zero;
use ledger::scan_state::currency::{Amount, Fee, Length, Magnitude, Nonce, Signed, Slot};
use ledger::scan_state::transaction_logic::protocol_state::{
    protocol_state_view, EpochData, EpochLedger, ProtocolStateView,
};
use ledger::scan_state::transaction_logic::transaction_applied::{
    signed_command_applied, CommandApplied, TransactionApplied, Varying,
};
use ledger::scan_state::transaction_logic::{
    apply_transactions, Transaction, TransactionStatus, UserCommand,
};
use ledger::sparse_ledger::LedgerIntf;
use ledger::staged_ledger::staged_ledger::StagedLedger;
use ledger::{dummy, Account, AccountId, Database, Mask, Timing, TokenId};
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
use openmina_core::constants::ConstraintConstants;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use ring_buffer::RingBuffer;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fs, str::FromStr};

/// Same values when we run `dune runtest src/lib/staged_ledger -f`
pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: 720000000000,
    supercharged_coinbase_factor: 2,
    account_creation_fee: 1000000000,
    fork: None,
};

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

// #[derive(BinProtWrite, Debug)]
// pub struct TxProofCreateInputs {
//     pub sok_message: MinaBaseSokMessageStableV1,
//     pub snarked_ledger_state: MinaStateSnarkedLedgerStateStableV2,
//     pub witness: TxWitness,
// }

// #[derive(BinProtWrite, Debug)]
// pub struct TxWitness {
//     pub transaction: Tx,
//     pub first_pass_ledger: MinaBaseSparseLedgerBaseStableV2,
//     pub second_pass_ledger: MinaBaseSparseLedgerBaseStableV2,
//     pub protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
//     pub init_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
//     pub status: MinaBaseTransactionStatusStableV2,
//     pub block_global_slot: UnsignedExtendedUInt32StableV1,
// }

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
                    let previous_hash = (&previous_hash.0)
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

// // TODO: remove this type once `Transaction` implements `BinProtWrite`.
// #[derive(BinProtWrite, Debug, Clone)]
// pub enum Tx {
//     UserCommand(UserCommand),
// }

// impl From<Tx> for Transaction {
//     fn from(value: Tx) -> Self {
//         match value {
//             Tx::UserCommand(v) => Self::Command(v),
//         }
//     }
// }

// impl From<Transaction> for Tx {
//     fn from(value: Transaction) -> Self {
//         match value {
//             Transaction::Command(v) => Tx::UserCommand(v),
//             _ => unimplemented!(),
//         }
//     }
// }

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
    pub fuzzcases_path: String,
    pub gen: GeneratorCtx,
    pub state: FuzzerState,
    pub snapshots: RingBuffer<FuzzerState>,
}

impl FuzzerCtx {
    #[coverage(off)]
    fn get_ledger_inner(&self) -> &Mask {
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

    // #[coverage(off)]
    // fn set_snarked_ledger(&mut self, snarked_ledger: Mask) {
    //     match &mut self.state.ledger {
    //         LedgerKind::Mask(_) => panic!(),
    //         LedgerKind::Staged(_, old_snarked_ledger) => *old_snarked_ledger = snarked_ledger,
    //     }
    // }

    // #[coverage(off)]
    // fn get_staged_ledger(&mut self) -> &mut StagedLedger {
    //     match &mut self.state.ledger {
    //         LedgerKind::Staged(ledger, _) => ledger,
    //         _ => panic!(),
    //     }
    // }

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

                    let permission_model = self.gen();
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
    pub fn get_account(&mut self, pkey: &CompressedPubKey) -> Option<Account> {
        let account_location = LedgerIntf::location_of_account(
            self.get_ledger_inner(),
            &AccountId::new(pkey.clone(), TokenId::default()),
        );

        account_location.map(
            #[coverage(off)]
            |location| *(LedgerIntf::get(self.get_ledger_inner(), &location).unwrap()).clone(),
        )
    }

    #[coverage(off)]
    pub fn find_sender(&mut self, pkey: &CompressedPubKey) -> Option<&(Keypair, PermissionModel)> {
        self.state.potential_senders.iter().find(
            #[coverage(off)]
            |(kp, _)| kp.public.into_compressed() == *pkey,
        )
    }

    #[coverage(off)]
    pub fn find_permissions(&mut self, pkey: &CompressedPubKey) -> Option<&PermissionModel> {
        self.find_sender(pkey).map(
            #[coverage(off)]
            |(_, pm)| pm,
        )
    }

    #[coverage(off)]
    pub fn find_keypair(&mut self, pkey: &CompressedPubKey) -> Option<&Keypair> {
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

    // #[coverage(off)]
    // pub fn random_create_tx_proof_inputs<F>(
    //     &mut self,
    //     protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
    //     mut verify_tx: F,
    // ) -> Option<(bool, TxProofCreateInputs)>
    // where
    //     F: FnMut(&Transaction) -> bool,
    // {
    //     let state_body_hash = MinaHash::hash(&protocol_state_body);
    //     let block_global_slot = protocol_state_body
    //         .consensus_state
    //         .global_slot_since_genesis
    //         .clone();
    //     let init_stack = protocol_state_body
    //         .blockchain_state
    //         .ledger_proof_statement
    //         .source
    //         .pending_coinbase_stack
    //         .clone();

    //     let tx = self.random_transaction();
    //     let is_valid = verify_tx(&tx);
    //     let transaction = WithStatus::applied(tx.clone());
    //     let tx = Tx::from(tx);

    //     let ledger = self.get_ledger_inner().make_child();
    //     let staged_ledger =
    //         StagedLedger::create_exn(self.constraint_constants.clone(), ledger).unwrap();
    //     let apply_res = StagedLedger::update_ledger_and_get_statements(
    //         &self.constraint_constants,
    //         // TODO(binier): construct from passed protocol_state_body.
    //         self.txn_state_view.global_slot_since_genesis,
    //         staged_ledger.ledger(),
    //         &(&init_stack).into(),
    //         (vec![transaction.clone()], None),
    //         // TODO(binier): construct from passed protocol_state_body.
    //         &self.txn_state_view,
    //         (
    //             // TODO(binier): use state hash instead. Not used anyways though.
    //             state_body_hash,
    //             state_body_hash,
    //         ),
    //     );
    //     let tx_with_witness = match apply_res {
    //         Ok((txs_with_witness, ..)) => txs_with_witness.into_iter().next().unwrap(),
    //         Err(_) => return None,
    //     };

    //     Some((
    //         is_valid,
    //         TxProofCreateInputs {
    //             sok_message: serde_json::from_value(serde_json::json!({
    //                 "fee":"25000000",
    //                 "prover":"B62qn7G9oFofQDGiAoP8TmYce7185PjWJ39unqjr2v7EgsRDoFCFc1k"
    //             }))
    //             .unwrap(),
    //             snarked_ledger_state: (&tx_with_witness.statement).into(),
    //             witness: TxWitness {
    //                 transaction: tx,
    //                 first_pass_ledger: (&tx_with_witness.first_pass_ledger_witness).into(),
    //                 second_pass_ledger: (&tx_with_witness.second_pass_ledger_witness).into(),
    //                 protocol_state_body,
    //                 // TODO(binier): should we somehow use value from `tx_with_witness`?
    //                 init_stack,
    //                 status: MinaBaseTransactionStatusStableV2::Applied,
    //                 block_global_slot,
    //             },
    //         },
    //     ))
    // }

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

    // #[coverage(off)]
    // pub fn serialize_transaction(tx: &Transaction) -> Vec<u8> {
    //     /*
    //             We don't have generated types for Transaction, but we have one
    //             for UserCommand (MinaBaseUserCommandStableV2). Extract and
    //             serialize the inner UserCommand and let a OCaml wrapper build
    //             the transaction.
    //     */
    //     match &tx {
    //         Transaction::Command(user_command) => serialize(user_command),
    //         _ => unimplemented!(),
    //     }
    // }

    // #[coverage(off)]
    // pub fn serialize_ledger(&self) -> Vec<u8> {
    //     serialize(&self.get_ledger_accounts())
    // }

    #[coverage(off)]
    fn save_fuzzcase(&self, tx: &Transaction, filename: &String) {
        let filename = self.fuzzcases_path.clone() + &filename + ".fuzzcase";

        println!("Saving fuzzcase: {}", filename);

        let user_command = match tx {
            Transaction::Command(user_command) => user_command.clone(),
            _ => unimplemented!(),
        };

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(filename)
            .unwrap();

        serialize(&(self.get_ledger_accounts(), user_command), &mut file);
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

    // #[coverage(off)]
    // pub fn apply_staged_ledger_diff(
    //     &mut self,
    //     diff: Diff,
    //     global_slot: Slot,
    //     coinbase_receiver: CompressedPubKey,
    //     current_view: ProtocolStateView,
    //     state_hashes: (Fp, Fp),
    //     state_tbl: &HashMap<Fp, MinaStateProtocolStateValueStableV2>,
    //     iteration: usize,
    // ) -> Result<StagedLedgerHash<Fp>, ()> {
    //     if iteration == 1271 {
    //         #[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
    //         struct State {
    //             scan_state: mina_p2p_messages::v2::TransactionSnarkScanStateStableV2,
    //             pending_coinbase_collection: mina_p2p_messages::v2::MinaBasePendingCoinbaseStableV2,
    //             states: Vec<(
    //                 mina_p2p_messages::bigint::BigInt,
    //                 mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2,
    //             )>,
    //             snarked_ledger: Vec<mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2>,
    //             expected_staged_ledger_merkle_root: mina_p2p_messages::bigint::BigInt,
    //         }

    //         let sl = self.get_staged_ledger();
    //         let sc = sl.scan_state.clone();
    //         let pcc = sl.pending_coinbase_collection.clone();
    //         let expected_staged_ledger_merkle_root = sl.ledger().clone().merkle_root();
    //         let snarked_ledger = self.get_snarked_ledger();

    //         let state = State {
    //             scan_state: (&sc).into(),
    //             pending_coinbase_collection: (&pcc).into(),
    //             states: state_tbl
    //                 .iter()
    //                 .map(|(h, v)| (h.into(), v.clone()))
    //                 .collect(),
    //             snarked_ledger: {
    //                 snarked_ledger
    //                     .to_list()
    //                     .into_iter()
    //                     .map(Into::into)
    //                     .collect()
    //                 // todo!()
    //             },
    //             expected_staged_ledger_merkle_root: expected_staged_ledger_merkle_root.into(),
    //         };

    //         let mut file = std::fs::File::create("/tmp/state.bin").unwrap();
    //         BinProtWrite::binprot_write(&state, &mut file).unwrap();
    //         file.sync_all().unwrap();

    //         eprintln!("data saved");
    //     }

    //     let constraint_constants = self.constraint_constants.clone();

    //     let DiffResult {
    //         hash_after_applying,
    //         ledger_proof,
    //         pending_coinbase_update: _,
    //     } = self
    //         .get_staged_ledger()
    //         .apply(
    //             None,
    //             &constraint_constants,
    //             global_slot,
    //             diff,
    //             (),
    //             &Verifier,
    //             &current_view,
    //             state_hashes,
    //             coinbase_receiver,
    //             false,
    //         )
    //         .map_err(|_| ())?;
    //     // .unwrap();

    //     if let Some((proof, _transactions)) = ledger_proof {
    //         self.update_snarked_ledger(state_tbl, proof)
    //     };

    //     self.get_staged_ledger().commit_and_reparent_to_root();

    //     Ok(hash_after_applying)
    // }

    // #[coverage(off)]
    // fn update_snarked_ledger(
    //     &mut self,
    //     state_tbl: &HashMap<Fp, MinaStateProtocolStateValueStableV2>,
    //     proof: LedgerProof,
    // ) {
    //     let target_snarked_ledger = {
    //         let stmt = proof.statement_ref();
    //         stmt.target.first_pass_ledger
    //     };

    //     let apply_first_pass = |global_slot: Slot,
    //                             txn_state_view: &ProtocolStateView,
    //                             ledger: &mut Mask,
    //                             transaction: &Transaction| {
    //         apply_transaction_first_pass(
    //             &CONSTRAINT_CONSTANTS,
    //             global_slot,
    //             txn_state_view,
    //             ledger,
    //             transaction,
    //         )
    //     };

    //     let apply_second_pass = |ledger: &mut Mask, tx: TransactionPartiallyApplied<Mask>| {
    //         apply_transaction_second_pass(&CONSTRAINT_CONSTANTS, ledger, tx)
    //     };

    //     let apply_first_pass_sparse_ledger =
    //         |global_slot: Slot,
    //          txn_state_view: &ProtocolStateView,
    //          sparse_ledger: &mut SparseLedger,
    //          transaction: &Transaction| {
    //             apply_transaction_first_pass(
    //                 &CONSTRAINT_CONSTANTS,
    //                 global_slot,
    //                 txn_state_view,
    //                 sparse_ledger,
    //                 transaction,
    //             )
    //         };

    //     let mut ledger = self.get_snarked_ledger().fuzzing_to_root();

    //     let get_state = |hash: Fp| Ok(state_tbl.get(&hash).cloned().unwrap());

    //     assert!(self
    //         .get_staged_ledger()
    //         .scan_state()
    //         .latest_ledger_proof()
    //         .is_some());

    //     self.get_staged_ledger()
    //         .scan_state()
    //         .get_snarked_ledger_sync(
    //             &mut ledger,
    //             get_state,
    //             apply_first_pass,
    //             apply_second_pass,
    //             apply_first_pass_sparse_ledger,
    //         )
    //         .unwrap();

    //     eprintln!("#############################################################");
    //     eprintln!("   NEW SNARKED LEDGER: {:?}", target_snarked_ledger);
    //     eprintln!("#############################################################");

    //     assert_eq!(ledger.merkle_root(), target_snarked_ledger);
    //     self.set_snarked_ledger(ledger);
    //     assert_eq!(
    //         self.get_snarked_ledger().merkle_root(),
    //         target_snarked_ledger
    //     );
    // }

    // #[coverage(off)]
    // pub fn create_staged_ledger_diff(
    //     &mut self,
    //     txns: Vec<Transaction>,
    //     global_slot: Slot,
    //     prover: CompressedPubKey,
    //     coinbase_receiver: CompressedPubKey,
    //     current_view: ProtocolStateView,
    //     ocaml_result: &Result<
    //         (
    //             StagedLedgerDiffDiffStableV2,
    //             Vec<(transaction_logic::valid::UserCommand, String)>,
    //         ),
    //         String,
    //     >,
    //     iteration: usize,
    //     snark_worker_fees: &mut Vec<Fee>,
    // ) -> Result<Option<Diff>, ()> {
    //     eprintln!();
    //     eprintln!("###################################################");
    //     eprintln!("              CREATE_STAGED_LEDGER_DIFF            ");
    //     eprintln!("###################################################");

    //     eprintln!(
    //         "get_staged_ledger num_account={:?}",
    //         self.get_staged_ledger().ledger.account_locations().len()
    //     );
    //     // get_staged_ledger
    //     self.gen.nonces.clear();

    //     let stmt_to_work_random_prover = |stmt: &Statement| -> Option<Checked> {
    //         let fee = snark_worker_fees.pop().unwrap();
    //         Some(Checked {
    //             fee,
    //             proofs: stmt.map(|statement| {
    //                 LedgerProof::create(
    //                     statement.clone(),
    //                     SokDigest::default(),
    //                     dummy::dummy_transaction_proof(),
    //                 )
    //             }),
    //             prover: prover.clone(),
    //         })
    //     };

    //     let txns = txns
    //         .into_iter()
    //         .map(|tx| {
    //             let Transaction::Command(cmd) = tx else {
    //                 unreachable!()
    //             };
    //             cmd.to_valid()
    //         })
    //         .collect();

    //     let constraint_constants = self.constraint_constants.clone();

    //     dbg!(global_slot);

    //     let result = self.get_staged_ledger().create_diff(
    //         &constraint_constants,
    //         global_slot,
    //         None,
    //         coinbase_receiver,
    //         (),
    //         &current_view,
    //         txns,
    //         stmt_to_work_random_prover,
    //         false, // Always false on berkeleynet now
    //     );

    //     // FIXME: ignoring error messages
    //     if result.is_err() && ocaml_result.is_err() {
    //         return Ok(None);
    //     }

    //     if !(result.is_ok() && ocaml_result.is_ok()) {
    //         println!(
    //             "!!! create_staged_ledger_diff mismatch between OCaml and Rust (result is_ok)\n{:?}\n{:?}\n",
    //             result, ocaml_result
    //         );

    //         //let bigint: num_bigint::BigUint = ledger.merkle_root().into();
    //         //self.save_fuzzcase(tx, &bigint.to_string());
    //         return Err(());
    //     }

    //     let (diff, invalid_cmds) = result.unwrap();
    //     let (ocaml_diff, ocaml_invalid_cmds) = ocaml_result.as_ref().unwrap();

    //     if iteration == 1271 {
    //         let mut file = std::fs::File::create("/tmp/diff.bin").unwrap();
    //         BinProtWrite::binprot_write(ocaml_diff, &mut file).unwrap();
    //         file.sync_all().unwrap();
    //         eprintln!("SAVED DIFF");
    //     }

    //     let diff = diff.forget();

    //     // FIXME: ignore error messages as work around for differences in string formatting between Rust and OCaml
    //     let rust_invalid_cmds: Vec<_> = invalid_cmds.iter().map(|x| x.0.clone()).collect();

    //     let ocaml_invalid_cmds2: Vec<_> = ocaml_invalid_cmds.iter().map(|x| x.0.clone()).collect();

    //     // Make sure we got same result
    //     if !(rust_invalid_cmds == ocaml_invalid_cmds2) {
    //         println!(
    //             "!!! create_staged_ledger_diff mismatch between OCaml and Rust (invalids)\n{}\n",
    //             self.diagnostic(&rust_invalid_cmds, &ocaml_invalid_cmds2)
    //         );

    //         eprintln!("last_string={:?}", ocaml_invalid_cmds.last().unwrap().1);

    //         //let bigint: num_bigint::BigUint = ledger.merkle_root().into();
    //         //self.save_fuzzcase(tx, &bigint.to_string());
    //         return Err(());
    //     }

    //     let ocaml_diff: Diff = ocaml_diff.into();

    //     if !(diff == ocaml_diff) {
    //         println!(
    //             "!!! create_staged_ledger_diff mismatch between OCaml and Rust (diff)\n{}\n",
    //             self.diagnostic(&diff, &ocaml_diff)
    //         );
    //         println!("!!! OCAML=\n{:?}\n", &ocaml_diff,);

    //         //let bigint: num_bigint::BigUint = ledger.merkle_root().into();
    //         //self.save_fuzzcase(tx, &bigint.to_string());
    //         return Err(());
    //     }

    //     Ok(Some(diff))
    // }

    // #[coverage(off)]
    // pub fn of_scan_state_pending_coinbases_and_snarked_ledger(
    //     &mut self,
    //     current_state: &MinaStateProtocolStateValueStableV2,
    //     state_tbl: &HashMap<Fp, MinaStateProtocolStateValueStableV2>,
    //     iteration: usize,
    // ) {
    //     eprintln!("#######################################################");
    //     eprintln!("of_scan_state_pending_coinbases_and_snarked_ledger");
    //     eprintln!("#######################################################");

    //     let get_state = |hash: Fp| state_tbl.get(&hash).cloned().unwrap();

    //     let mut snarked_ledger = self.get_snarked_ledger().fuzzing_to_root();
    //     let sl = self.get_staged_ledger();
    //     let expected_hash: StagedLedgerHash = sl.hash();
    //     let expected_staged_ledger_merkle_root = sl.ledger.clone().merkle_root();

    //     dbg!(snarked_ledger.merkle_root());

    //     let new_staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
    //         (),
    //         &CONSTRAINT_CONSTANTS,
    //         Verifier,
    //         sl.scan_state.clone(),
    //         snarked_ledger.copy(),
    //         {
    //             let registers: transaction_snark::Registers = (&current_state
    //                 .body
    //                 .blockchain_state
    //                 .ledger_proof_statement
    //                 .target)
    //                 .into();
    //             registers.local_state
    //         },
    //         expected_staged_ledger_merkle_root,
    //         sl.pending_coinbase_collection.clone(),
    //         get_state,
    //     );

    //     // if new_staged_ledger.is_err() || iteration == 370 {

    //     //     #[derive(Clone, Debug, PartialEq, binprot_derive::BinProtRead, BinProtWrite)]
    //     //     struct State {
    //     //         scan_state: mina_p2p_messages::v2::TransactionSnarkScanStateStableV2,
    //     //         pending_coinbase_collection: mina_p2p_messages::v2::MinaBasePendingCoinbaseStableV2,
    //     //         states: Vec<(mina_p2p_messages::bigint::BigInt, MinaStateProtocolStateValueStableV2)>,
    //     //         snarked_ledger: Vec<mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2>,
    //     //         expected_staged_ledger_merkle_root: mina_p2p_messages::bigint::BigInt,
    //     //     }

    //     //     let sc = sl.scan_state.clone();
    //     //     let pcc = sl.pending_coinbase_collection.clone();

    //     //     let state = State {
    //     //         scan_state: (&sc).into(),
    //     //         pending_coinbase_collection: (&pcc).into(),
    //     //         states: state_tbl.iter().map(|(h, v)| (h.into(), v.clone())).collect(),
    //     //         snarked_ledger: {
    //     //             use crate::BaseLedger;
    //     //             snarked_ledger.to_list().into_iter().map(Into::into).collect()
    //     //             // todo!()
    //     //         },
    //     //         expected_staged_ledger_merkle_root: expected_staged_ledger_merkle_root.into(),
    //     //     };

    //     //     let mut file = std::fs::File::create("/tmp/state.bin").unwrap();
    //     //     BinProtWrite::binprot_write(&state, &mut file).unwrap();
    //     //     file.sync_all().unwrap();

    //     //     eprintln!("data saved");
    //     // }

    //     let mut new_staged_ledger = new_staged_ledger.unwrap();

    //     assert_eq!(expected_hash, sl.hash());
    //     assert_eq!(expected_hash, new_staged_ledger.hash());
    //     eprintln!("#######################################################");
    //     eprintln!("of_scan_state_pending_coinbases_and_snarked_ledger OK");
    //     eprintln!("#######################################################");
    // }

    #[coverage(off)]
    fn diagnostic(&self, applied: &impl Debug, applied_ocaml: &impl Debug) -> String {
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
    pub fn apply_transaction(
        &mut self,
        user_command: &UserCommand,
        expected_apply_result: &ApplyTxResult,
    ) -> Result<(), ()> {
        self.gen.nonces.clear();

        let mut ledger = self.get_ledger_inner().make_child();

        // If we called apply_transaction it means we passed the tx pool check, so add tx to the cache
        if let UserCommand::ZkAppCommand(command) = user_command {
            if !command.account_updates.is_empty() {
                //println!("Storing in pool cache {:?}", tx);
                self.state.cache_pool.push_back(user_command.clone());
            }
        }

        //println!("tx: {:?}\n", tx);
        let tx = Transaction::Command(user_command.clone());

        let applied = apply_transactions(
            &self.constraint_constants,
            self.txn_state_view.global_slot_since_genesis,
            &self.txn_state_view,
            &mut ledger,
            &[tx.clone()],
        );

        // println!(
        //     "tx: {:?}\n applied: {:?}\n expected: {:?}",
        //     tx, applied, expected_apply_result
        // );

        match applied {
            Ok(applied) => {
                // For now we work with one transaction at a time
                let applied = &applied[0];

                if expected_apply_result.apply_result.len() != 1 {
                    println!(
                        "!!! Apply failed in OCaml (error: {}) but it didn't in Rust: {:?}",
                        expected_apply_result.error, applied
                    );
                    let bigint: num_bigint::BigUint = LedgerIntf::merkle_root(&mut ledger).into();
                    self.save_fuzzcase(&tx, &bigint.to_string());
                    return Err(());
                } else {
                    if applied != &expected_apply_result.apply_result[0] {
                        println!(
                            "!!! Apply result mismatch between OCaml and Rust\n{}\n",
                            self.diagnostic(applied, &expected_apply_result.apply_result[0])
                        );

                        let bigint: num_bigint::BigUint =
                            LedgerIntf::merkle_root(&mut ledger).into();
                        self.save_fuzzcase(&tx, &bigint.to_string());
                        return Err(());
                    }
                }

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
                    let bigint: num_bigint::BigUint = LedgerIntf::merkle_root(&mut ledger).into();
                    self.save_fuzzcase(&tx, &bigint.to_string());
                    return Err(());
                }

                if expected_apply_result.apply_result.len() == 1 {
                    println!(
                        "!!! Apply failed in Rust (error: {}) but it didn't in OCaml: {:?}",
                        error_string, &expected_apply_result.apply_result[0]
                    );
                    let bigint: num_bigint::BigUint = LedgerIntf::merkle_root(&mut ledger).into();
                    self.save_fuzzcase(&tx, &bigint.to_string());
                    return Err(());
                }
            }
        }

        let rust_ledger_root_hash = LedgerIntf::merkle_root(&mut ledger);

        if &expected_apply_result.root_hash != &rust_ledger_root_hash {
            println!(
                "Ledger hash mismatch: {:?} != {:?} (expected)",
                rust_ledger_root_hash, expected_apply_result.root_hash
            );
            let bigint: num_bigint::BigUint = rust_ledger_root_hash.into();
            self.save_fuzzcase(&tx, &bigint.to_string());
            Err(())
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
    fuzzcases_path: Option<String>,
    seed: u64,
    minimum_fee: u64,
    max_account_balance: u64,
    initial_accounts: usize,
    cache_size: usize,
    snapshots_size: usize,
    is_staged_ledger: bool,
}

impl FuzzerCtxBuilder {
    #[coverage(off)]
    pub fn new() -> Self {
        Self {
            constraint_constants: None,
            txn_state_view: None,
            fuzzcases_path: None,
            seed: 0,
            minimum_fee: 1_000_000, // Sane default in case we don't obtain it from OCaml
            max_account_balance: 1_000_000_000_000_000,
            initial_accounts: 10,
            cache_size: 4096,
            snapshots_size: 128,
            is_staged_ledger: false,
        }
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
        let constraint_constants = self
            .constraint_constants
            .clone()
            .unwrap_or(CONSTRAINT_CONSTANTS);
        let depth = constraint_constants.ledger_depth as usize;
        let root = Mask::new_root(Database::create(depth.try_into().unwrap()));
        let txn_state_view = self
            .txn_state_view
            .clone()
            .unwrap_or(dummy_state_view(None));
        let fuzzcases_path = self.fuzzcases_path.clone().unwrap_or("./".to_string());

        let ledger = match self.is_staged_ledger {
            true => {
                let snarked_ledger_mask = root.make_child().fuzzing_to_root();
                // let snarked_ledger_mask = root.make_child();
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
            },
            snapshots: RingBuffer::with_capacity(self.snapshots_size),
        };

        ctx.create_inital_accounts(self.initial_accounts);
        ctx
    }
}

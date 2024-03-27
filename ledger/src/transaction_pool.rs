use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    sync::Arc,
};

use itertools::Itertools;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint::BigInt, v2};
use openmina_core::constants::{ForkConstants, CONSTRAINT_CONSTANTS};

use crate::{
    scan_state::{
        currency::{Amount, Balance, BlockTime, Fee, Magnitude, Nonce, Slot, SlotSpan},
        fee_rate::FeeRate,
        transaction_logic::{
            valid,
            zkapp_command::{
                from_unapplied_sequence::{self, FromUnappliedSequence},
                MaybeWithStatus, WithHash,
            },
            TransactionStatus::Applied,
            UserCommand, WithStatus,
        },
    },
    verifier::Verifier,
    Account, AccountId, BaseLedger, Mask, TokenId, VerificationKey,
};

mod consensus {
    use crate::scan_state::currency::{BlockTimeSpan, Epoch, Length};

    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Constants {
        k: Length,
        delta: Length,
        slots_per_sub_window: Length,
        slots_per_window: Length,
        sub_windows_per_window: Length,
        slots_per_epoch: Length,
        grace_period_slots: Length,
        grace_period_end: Slot,
        checkpoint_window_slots_per_year: Length,
        checkpoint_window_size_in_slots: Length,
        block_window_duration_ms: BlockTimeSpan,
        slot_duration_ms: BlockTimeSpan,
        epoch_duration: BlockTimeSpan,
        delta_duration: BlockTimeSpan,
        genesis_state_timestamp: BlockTime,
    }

    // Consensus epoch
    impl Epoch {
        fn of_time_exn(constants: &Constants, time: BlockTime) -> Result<Self, String> {
            if time < constants.genesis_state_timestamp {
                return Err(
                    "Epoch.of_time: time is earlier than genesis block timestamp".to_string(),
                );
            }

            let time_since_genesis = time.diff(constants.genesis_state_timestamp);
            let epoch = time_since_genesis.to_ms() / constants.epoch_duration.to_ms();
            let epoch: u32 = epoch.try_into().unwrap();

            Ok(Self::from_u32(epoch))
        }

        fn start_time(constants: &Constants, epoch: Self) -> BlockTime {
            let ms = constants
                .genesis_state_timestamp
                .to_span_since_epoch()
                .to_ms()
                + ((epoch.as_u32() as u64) * constants.epoch_duration.to_ms());
            BlockTime::of_span_since_epoch(BlockTimeSpan::of_ms(ms))
        }

        pub fn epoch_and_slot_of_time_exn(
            constants: &Constants,
            time: BlockTime,
        ) -> Result<(Self, Slot), String> {
            let epoch = Self::of_time_exn(constants, time)?;
            let time_since_epoch = time.diff(Self::start_time(constants, epoch));

            let slot: u64 = time_since_epoch.to_ms() / constants.slot_duration_ms.to_ms();
            let slot = Slot::from_u32(slot.try_into().unwrap());

            Ok((epoch, slot))
        }
    }

    /// TODO: Maybe rename to `ConsensusGlobalSlot` ?
    pub struct GlobalSlot {
        slot_number: Slot,
        slots_per_epoch: Length,
    }

    impl GlobalSlot {
        fn create(constants: &Constants, epoch: Epoch, slot: Slot) -> Self {
            let slot_number = slot.as_u32() + (constants.slots_per_epoch.as_u32() * epoch.as_u32());
            Self {
                slot_number: Slot::from_u32(slot_number),
                slots_per_epoch: constants.slots_per_epoch,
            }
        }

        fn of_epoch_and_slot(constants: &Constants, (epoch, slot): (Epoch, Slot)) -> Self {
            Self::create(constants, epoch, slot)
        }

        pub fn of_time_exn(constants: &Constants, time: BlockTime) -> Result<Self, String> {
            Ok(Self::of_epoch_and_slot(
                constants,
                Epoch::epoch_and_slot_of_time_exn(constants, time)?,
            ))
        }

        pub fn to_global_slot(&self) -> Slot {
            let Self {
                slot_number,
                slots_per_epoch: _,
            } = self;
            *slot_number
        }
    }
}

/// Fee increase required to replace a transaction.
const REPLACE_FEE: Fee = Fee::of_nanomina_int_exn(1);

type ValidCommandWithHash = WithHash<valid::UserCommand, BlakeHash>;

pub mod diff {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum Error {
        InsufficientReplaceFee,
        Duplicate,
        InvalidNonce,
        InsufficientFunds,
        Overflow,
        BadToken,
        UnwantedFeeToken,
        Expired,
        Overloaded,
        FeePayerAccountNotFound,
        FeePayerNotPermittedToSend,
        AfterSlotTxEnd,
    }

    impl Error {
        pub fn grounds_for_diff_rejection(&self) -> bool {
            match self {
                Error::InsufficientReplaceFee
                | Error::Duplicate
                | Error::InvalidNonce
                | Error::InsufficientFunds
                | Error::Expired
                | Error::Overloaded
                | Error::FeePayerAccountNotFound
                | Error::FeePayerNotPermittedToSend
                | Error::AfterSlotTxEnd => false,
                Error::Overflow | Error::BadToken | Error::UnwantedFeeToken => true,
            }
        }
    }

    pub struct Diff {
        pub list: Vec<UserCommand>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DiffVerified {
        pub list: Vec<ValidCommandWithHash>,
    }

    struct Rejected {
        list: Vec<(UserCommand, Error)>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct BestTipDiff {
        pub new_commands: Vec<WithStatus<valid::UserCommand>>,
        pub removed_commands: Vec<WithStatus<valid::UserCommand>>,
        pub reorg_best_tip: bool,
    }
}

fn preload_accounts(
    ledger: &Mask,
    account_ids: &HashSet<AccountId>,
) -> HashMap<AccountId, Account> {
    account_ids
        .iter()
        .filter_map(|id| {
            let addr = ledger.location_of_account(id)?;
            let account = ledger.get(addr)?;
            Some((id.clone(), *account))
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    trust_system: (),
    pool_max_size: usize,
    slot_tx_end: Option<Slot>,
}

pub type VerificationKeyWire = WithHash<VerificationKey>;

/// Used to be able to de/serialize our `TransactionPool` in the state machine
#[derive(Serialize, Deserialize)]
struct VkRefcountTableBigInts {
    verification_keys: Vec<(BigInt, (usize, WithHash<VerificationKey, BigInt>))>,
    account_id_to_vks: Vec<(AccountId, Vec<(BigInt, usize)>)>,
    vk_to_account_ids: Vec<(BigInt, Vec<(AccountId, usize)>)>,
}
impl From<VkRefcountTable> for VkRefcountTableBigInts {
    fn from(value: VkRefcountTable) -> Self {
        let VkRefcountTable {
            verification_keys,
            account_id_to_vks,
            vk_to_account_ids,
        } = value;
        Self {
            verification_keys: verification_keys
                .into_iter()
                .map(|(hash, (count, vk))| {
                    assert_eq!(hash, vk.hash);
                    let hash: BigInt = hash.into();
                    (
                        hash.clone(),
                        (
                            count,
                            WithHash {
                                data: vk.data,
                                hash,
                            },
                        ),
                    )
                })
                .collect(),
            account_id_to_vks: account_id_to_vks
                .into_iter()
                .map(|(id, map)| {
                    (
                        id,
                        map.into_iter()
                            .map(|(hash, count)| (hash.into(), count))
                            .collect(),
                    )
                })
                .collect(),
            vk_to_account_ids: vk_to_account_ids
                .into_iter()
                .map(|(hash, map)| (hash.into(), map.into_iter().collect()))
                .collect(),
        }
    }
}
impl From<VkRefcountTableBigInts> for VkRefcountTable {
    fn from(value: VkRefcountTableBigInts) -> Self {
        let VkRefcountTableBigInts {
            verification_keys,
            account_id_to_vks,
            vk_to_account_ids,
        } = value;
        Self {
            verification_keys: verification_keys
                .into_iter()
                .map(|(hash, (count, vk))| {
                    assert_eq!(hash, vk.hash);
                    let hash: Fp = hash.to_field();
                    (
                        hash,
                        (
                            count,
                            WithHash {
                                data: vk.data,
                                hash,
                            },
                        ),
                    )
                })
                .collect(),
            account_id_to_vks: account_id_to_vks
                .into_iter()
                .map(|(id, map)| {
                    let map = map
                        .into_iter()
                        .map(|(bigint, count)| (bigint.to_field::<Fp>(), count))
                        .collect();
                    (id, map)
                })
                .collect(),
            vk_to_account_ids: vk_to_account_ids
                .into_iter()
                .map(|(hash, map)| (hash.to_field(), map.into_iter().collect()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(into = "VkRefcountTableBigInts")]
#[serde(from = "VkRefcountTableBigInts")]
struct VkRefcountTable {
    verification_keys: HashMap<Fp, (usize, VerificationKeyWire)>,
    account_id_to_vks: HashMap<AccountId, HashMap<Fp, usize>>,
    vk_to_account_ids: HashMap<Fp, HashMap<AccountId, usize>>,
}

impl VkRefcountTable {
    fn find_vk(&self, f: &Fp) -> Option<&(usize, WithHash<VerificationKey>)> {
        self.verification_keys.get(f)
    }

    fn find_vks_by_account_id(&self, account_id: &AccountId) -> Vec<&VerificationKeyWire> {
        let Some(vks) = self.account_id_to_vks.get(account_id) else {
            return Vec::new();
        };

        vks.iter()
            .map(|(f, _)| self.find_vk(f).expect("malformed Vk_refcount_table.t"))
            .map(|(_, vk)| vk)
            .collect()
    }

    fn inc(&mut self, account_id: AccountId, vk: VerificationKeyWire) {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        match self.verification_keys.entry(vk.hash) {
            Vacant(e) => {
                e.insert((1, vk.clone()));
            }
            Occupied(mut e) => {
                let (count, _vk) = e.get_mut();
                *count += 1;
            }
        }

        let map = self
            .account_id_to_vks
            .entry(account_id.clone())
            .or_default(); // or insert empty map
        let count = map.entry(vk.hash).or_default(); // or insert count 0
        *count += 1;

        let map = self.vk_to_account_ids.entry(vk.hash).or_default(); // or insert empty map
        let count = map.entry(account_id).or_default(); // or insert count 0
        *count += 1;
    }

    fn dec(&mut self, account_id: AccountId, vk_hash: Fp) {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        match self.verification_keys.entry(vk_hash) {
            Vacant(_e) => unreachable!(),
            Occupied(mut e) => {
                let (count, _vk) = e.get_mut();
                if *count == 1 {
                    e.remove();
                } else {
                    *count = (*count).checked_sub(1).unwrap();
                }
            }
        }

        fn remove<K1, K2>(key1: K1, key2: K2, table: &mut HashMap<K1, HashMap<K2, usize>>)
        where
            K1: std::hash::Hash + Eq,
            K2: std::hash::Hash + Eq,
        {
            match table.entry(key1) {
                Vacant(_e) => unreachable!(),
                Occupied(mut e) => {
                    let map = e.get_mut();
                    match map.entry(key2) {
                        Vacant(_e) => unreachable!(),
                        Occupied(mut e2) => {
                            let count: &mut usize = e2.get_mut();
                            if *count == 1 {
                                e2.remove();
                                e.remove();
                            } else {
                                *count = count.checked_sub(1).expect("Invalid state");
                            }
                        }
                    }
                }
            }
        }

        remove(account_id.clone(), vk_hash, &mut self.account_id_to_vks);
        remove(vk_hash, account_id.clone(), &mut self.vk_to_account_ids);
    }

    fn decrement_list(&mut self, list: &[WithStatus<valid::UserCommand>]) {
        list.iter().for_each(|c| {
            for (id, vk) in c.data.forget_check().extract_vks() {
                self.dec(id, vk.hash);
            }
        });
    }

    fn decrement_hashed<I, V>(&mut self, list: I)
    where
        I: IntoIterator<Item = V>,
        V: Borrow<ValidCommandWithHash>,
    {
        list.into_iter().for_each(|c| {
            for (id, vk) in c.borrow().data.forget_check().extract_vks() {
                self.dec(id, vk.hash);
            }
        });
    }

    fn increment_hashed<I, V>(&mut self, list: I)
    where
        I: IntoIterator<Item = V>,
        V: Borrow<ValidCommandWithHash>,
    {
        list.into_iter().for_each(|c| {
            for (id, vk) in c.borrow().data.forget_check().extract_vks() {
                self.inc(id, vk);
            }
        });
    }

    fn increment_list(&mut self, list: &[WithStatus<valid::UserCommand>]) {
        list.iter().for_each(|c| {
            for (id, vk) in c.data.forget_check().extract_vks() {
                self.inc(id, vk);
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Batch {
    Of(usize),
}

pub enum CommandError {
    InvalidNonce {
        account_nonce: Nonce,
        expected: Nonce,
    },
    InsufficientFunds {
        balance: Balance,
        consumed: Amount,
    },
    /// NOTE: don't punish for this, attackers can induce nodes to banlist
    ///       each other that way! *)
    InsufficientReplaceFee {
        replace_fee: Fee,
        fee: Fee,
    },
    Overflow,
    BadToken,
    Expired {
        valid_until: Slot,
        global_slot_since_genesis: Slot,
    },
    UnwantedFeeToken {
        token_id: TokenId,
    },
    AfterSlotTxEnd,
}

impl From<CommandError> for diff::Error {
    fn from(value: CommandError) -> Self {
        match value {
            CommandError::InvalidNonce { .. } => diff::Error::InvalidNonce,
            CommandError::InsufficientFunds { .. } => diff::Error::InsufficientFunds,
            CommandError::InsufficientReplaceFee { .. } => diff::Error::InsufficientReplaceFee,
            CommandError::Overflow => diff::Error::Overflow,
            CommandError::BadToken => diff::Error::BadToken,
            CommandError::Expired { .. } => diff::Error::Expired,
            CommandError::UnwantedFeeToken { .. } => diff::Error::UnwantedFeeToken,
            CommandError::AfterSlotTxEnd => diff::Error::AfterSlotTxEnd,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IndexedPoolConfig {
    consensus_constants: consensus::Constants,
    slot_tx_end: Option<Slot>,
}

// module Config = struct
//   type t =
//     { constraint_constants : Genesis_constants.Constraint_constants.t
//     ; consensus_constants : Consensus.Constants.t
//     ; time_controller : Block_time.Controller.t
//     ; slot_tx_end : Mina_numbers.Global_slot_since_hard_fork.t option
//     }
//   [@@deriving sexp_of, equal, compare]
// end

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IndexedPool {
    /// Transactions valid against the current ledger, indexed by fee per
    /// weight unit.
    applicable_by_fee: HashMap<FeeRate, HashSet<ValidCommandWithHash>>,
    /// All pending transactions along with the total currency required to
    /// execute them -- plus any currency spent from this account by
    /// transactions from other accounts -- indexed by sender account.
    /// Ordered by nonce inside the accounts.
    all_by_sender: HashMap<AccountId, (VecDeque<ValidCommandWithHash>, Amount)>,
    /// All transactions in the pool indexed by fee per weight unit.
    all_by_fee: HashMap<FeeRate, HashSet<ValidCommandWithHash>>,
    all_by_hash: HashMap<BlakeHash, ValidCommandWithHash>,
    /// Only transactions that have an expiry
    transactions_with_expiration: HashMap<Slot, HashSet<ValidCommandWithHash>>,
    size: usize,
    config: IndexedPoolConfig,
}

enum Update {
    Add {
        command: ValidCommandWithHash,
        fee_per_wu: FeeRate,
        add_to_applicable_by_fee: bool,
    },
    RemoveAllByFeeAndHashAndExpiration {
        commands: VecDeque<ValidCommandWithHash>,
    },
    RemoveFromApplicableByFee {
        fee_per_wu: FeeRate,
        command: ValidCommandWithHash,
    },
}

#[derive(Clone)]
struct SenderState {
    sender: AccountId,
    state: Option<(VecDeque<ValidCommandWithHash>, Amount)>,
}

pub enum RevalidateKind<'a> {
    EntirePool,
    Subset(&'a HashSet<AccountId>),
}

impl IndexedPool {
    fn size(&self) -> usize {
        self.size
    }

    fn min_fee(&self) -> Option<FeeRate> {
        self.all_by_fee.keys().min().cloned()
    }

    fn member(&self, cmd: &ValidCommandWithHash) -> bool {
        self.all_by_hash.contains_key(&cmd.hash)
    }

    fn global_slot_since_genesis(&self) -> Slot {
        let current_time = BlockTime::now();

        let current_slot =
            consensus::GlobalSlot::of_time_exn(&self.config.consensus_constants, current_time)
                .unwrap()
                .to_global_slot();

        match CONSTRAINT_CONSTANTS.fork.as_ref() {
            Some(ForkConstants { genesis_slot, .. }) => {
                let genesis_slot = Slot::from_u32(*genesis_slot);
                let slot_span = SlotSpan::from_u32(current_slot.as_u32());
                genesis_slot.add(slot_span)
            }
            None => current_slot,
        }
    }

    fn check_expiry(&self, cmd: &UserCommand) -> Result<(), CommandError> {
        let global_slot_since_genesis = self.global_slot_since_genesis();
        let valid_until = cmd.valid_until();

        if valid_until < global_slot_since_genesis {
            return Err(CommandError::Expired {
                valid_until,
                global_slot_since_genesis,
            });
        }

        Ok(())
    }

    /// Insert in a `HashMap<_, HashSet<_>>`
    fn map_set_insert<K, V>(map: &mut HashMap<K, HashSet<V>>, key: K, value: V)
    where
        K: std::hash::Hash + PartialEq + Eq,
        V: std::hash::Hash + PartialEq + Eq,
    {
        let entry = map.entry(key).or_default();
        entry.insert(value);
    }

    /// Remove in a `HashMap<_, HashSet<_>>`
    fn map_set_remove<K, V>(map: &mut HashMap<K, HashSet<V>>, key: K, value: &V)
    where
        K: std::hash::Hash + PartialEq + Eq,
        V: std::hash::Hash + PartialEq + Eq,
    {
        let Entry::Occupied(mut entry) = map.entry(key) else {
            return;
        };
        let set = entry.get_mut();
        set.remove(value);
        if set.is_empty() {
            entry.remove();
        }
    }

    fn update_expiration_map(&mut self, cmd: ValidCommandWithHash, is_add: bool) {
        let user_cmd = cmd.data.forget_check();
        let expiry = user_cmd.valid_until();
        if expiry == Slot::max() {
            return; // Do nothing
        }
        if is_add {
            Self::map_set_insert(&mut self.transactions_with_expiration, expiry, cmd);
        } else {
            Self::map_set_remove(&mut self.transactions_with_expiration, expiry, &cmd);
        }
    }

    fn remove_from_expiration_exn(&mut self, cmd: ValidCommandWithHash) {
        self.update_expiration_map(cmd, false);
    }

    fn add_to_expiration(&mut self, cmd: ValidCommandWithHash) {
        self.update_expiration_map(cmd, true);
    }

    /// Remove a command from the applicable_by_fee field. This may break an
    /// invariant.
    fn remove_applicable_exn(&mut self, cmd: &ValidCommandWithHash) {
        let fee_per_wu = cmd.data.forget_check().fee_per_wu();
        Self::map_set_remove(&mut self.applicable_by_fee, fee_per_wu, cmd);
    }

    fn make_queue<T>() -> VecDeque<T> {
        VecDeque::with_capacity(256)
    }

    fn add_from_backtrack(&mut self, cmd: ValidCommandWithHash) -> Result<(), CommandError> {
        let IndexedPoolConfig { slot_tx_end, .. } = &self.config;

        let current_global_slot = self.current_global_slot();

        if !slot_tx_end
            .as_ref()
            .map(|slot_tx_end| current_global_slot < *slot_tx_end)
            .unwrap_or(true)
        {
            return Err(CommandError::AfterSlotTxEnd);
        }

        let ValidCommandWithHash {
            data: unchecked,
            hash: cmd_hash,
        } = &cmd;
        let unchecked = unchecked.forget_check();

        self.check_expiry(&unchecked)?;

        let fee_payer = unchecked.fee_payer();
        let fee_per_wu = unchecked.fee_per_wu();

        let consumed = currency_consumed(&unchecked).unwrap();

        match self.all_by_sender.get_mut(&fee_payer) {
            None => {
                {
                    let mut queue = Self::make_queue();
                    queue.push_back(cmd.clone());
                    self.all_by_sender.insert(fee_payer, (queue, consumed));
                }
                Self::map_set_insert(&mut self.all_by_fee, fee_per_wu.clone(), cmd.clone());
                self.all_by_hash.insert(cmd_hash.clone(), cmd.clone());
                Self::map_set_insert(&mut self.applicable_by_fee, fee_per_wu.clone(), cmd.clone());
                self.add_to_expiration(cmd);
                self.size += 1;
            }
            Some((queue, currency_reserved)) => {
                let first_queued = queue.front().cloned().unwrap();

                if unchecked.expected_target_nonce()
                    != first_queued.data.forget_check().applicable_at_nonce()
                {
                    panic!("indexed pool nonces inconsistent when adding from backtrack.")
                }

                // update `self.all_by_sender`
                {
                    queue.push_front(cmd.clone());
                    *currency_reserved = currency_reserved.checked_add(&consumed).unwrap();
                }

                self.remove_applicable_exn(&first_queued);

                Self::map_set_insert(&mut self.applicable_by_fee, fee_per_wu.clone(), cmd.clone());
                Self::map_set_insert(&mut self.all_by_fee, fee_per_wu.clone(), cmd.clone());
                self.all_by_hash.insert(cmd_hash.clone(), cmd.clone());
                self.add_to_expiration(cmd);
                self.size += 1;
            }
        }
        Ok(())
    }

    fn current_global_slot(&self) -> Slot {
        let IndexedPoolConfig {
            consensus_constants,
            slot_tx_end: _,
        } = &self.config;

        consensus::GlobalSlot::of_time_exn(consensus_constants, BlockTime::now())
            .unwrap()
            .to_global_slot()
    }

    fn update_add(
        &mut self,
        cmd: ValidCommandWithHash,
        fee_per_wu: FeeRate,
        add_to_applicable_by_fee: bool,
    ) {
        if add_to_applicable_by_fee {
            Self::map_set_insert(&mut self.applicable_by_fee, fee_per_wu.clone(), cmd.clone());
        }

        let cmd_hash = cmd.hash.clone();

        Self::map_set_insert(&mut self.all_by_fee, fee_per_wu, cmd.clone());
        self.all_by_hash.insert(cmd_hash, cmd.clone());
        self.add_to_expiration(cmd);
        self.size += 1;
    }

    /// Remove a command from the all_by_fee and all_by_hash fields, and decrement
    /// size. This may break an invariant.
    fn update_remove_all_by_fee_and_hash_and_expiration<I>(&mut self, cmds: I)
    where
        I: IntoIterator<Item = ValidCommandWithHash>,
    {
        for cmd in cmds {
            let fee_per_wu = cmd.data.forget_check().fee_per_wu();
            let cmd_hash = cmd.hash.clone();
            Self::map_set_remove(&mut self.all_by_fee, fee_per_wu, &cmd);
            self.all_by_hash.remove(&cmd_hash);
            self.remove_from_expiration_exn(cmd);
            self.size = self.size.checked_sub(1).unwrap();
        }
    }

    fn update_remove_from_applicable_by_fee(
        &mut self,
        fee_per_wu: FeeRate,
        command: &ValidCommandWithHash,
    ) {
        Self::map_set_remove(&mut self.applicable_by_fee, fee_per_wu, command)
    }

    fn remove_with_dependents_exn(
        &mut self,
        cmd: &ValidCommandWithHash,
    ) -> VecDeque<ValidCommandWithHash> {
        let sender = cmd.data.fee_payer();
        let mut by_sender = SenderState {
            state: self.all_by_sender.get(&sender).cloned(),
            sender,
        };

        let mut updates = Vec::<Update>::with_capacity(128);
        let result = self.remove_with_dependents_exn_impl(cmd, &mut by_sender, &mut updates);

        self.set_sender(by_sender);
        self.apply_updates(updates);

        result
    }

    fn remove_with_dependents_exn_impl(
        &self,
        cmd: &ValidCommandWithHash,
        by_sender: &mut SenderState,
        updates: &mut Vec<Update>,
    ) -> VecDeque<ValidCommandWithHash> {
        let (sender_queue, reserved_currency_ref) = by_sender.state.as_mut().unwrap();
        let unchecked = cmd.data.forget_check();

        assert!(!sender_queue.is_empty());

        let cmd_nonce = unchecked.applicable_at_nonce();

        let cmd_index = sender_queue
            .iter()
            .position(|cmd| {
                let nonce = cmd.data.forget_check().applicable_at_nonce();
                // we just compare nonce equality since the command we are looking for already exists in the sequence
                nonce == cmd_nonce
            })
            .unwrap();

        let drop_queue = sender_queue.split_off(cmd_index);
        let keep_queue = sender_queue;
        assert!(!drop_queue.is_empty());

        let currency_to_remove = drop_queue.iter().fold(Amount::zero(), |acc, cmd| {
            let consumed = currency_consumed(&cmd.data.forget_check()).unwrap();
            consumed.checked_add(&acc).unwrap()
        });

        // This is safe because the currency in a subset of the commands much be <=
        // total currency in all the commands.
        let reserved_currency = reserved_currency_ref
            .checked_sub(&currency_to_remove)
            .unwrap();

        updates.push(Update::RemoveAllByFeeAndHashAndExpiration {
            commands: drop_queue.clone(),
        });

        if cmd_index == 0 {
            updates.push(Update::RemoveFromApplicableByFee {
                fee_per_wu: unchecked.fee_per_wu(),
                command: cmd.clone(),
            });
        }

        // We re-fetch it to make the borrow checker happy
        // let (keep_queue, reserved_currency_ref) = self.all_by_sender.get_mut(&sender).unwrap();
        if !keep_queue.is_empty() {
            *reserved_currency_ref = reserved_currency;
        } else {
            assert!(reserved_currency.is_zero());
            by_sender.state = None;
        }

        drop_queue
    }

    fn apply_updates(&mut self, updates: Vec<Update>) {
        for update in updates {
            match update {
                Update::Add {
                    command,
                    fee_per_wu,
                    add_to_applicable_by_fee,
                } => self.update_add(command, fee_per_wu, add_to_applicable_by_fee),
                Update::RemoveAllByFeeAndHashAndExpiration { commands } => {
                    self.update_remove_all_by_fee_and_hash_and_expiration(commands)
                }
                Update::RemoveFromApplicableByFee {
                    fee_per_wu,
                    command,
                } => self.update_remove_from_applicable_by_fee(fee_per_wu, &command),
            }
        }
    }

    fn set_sender(&mut self, by_sender: SenderState) {
        let SenderState { sender, state } = by_sender;

        match state {
            Some(state) => {
                self.all_by_sender.insert(sender, state);
            }
            None => {
                self.all_by_sender.remove(&sender);
            }
        }
    }

    fn add_from_gossip_exn(
        &mut self,
        cmd: &ValidCommandWithHash,
        current_nonce: Nonce,
        balance: Balance,
    ) -> Result<(ValidCommandWithHash, VecDeque<ValidCommandWithHash>), CommandError> {
        let sender = cmd.data.fee_payer();
        let mut by_sender = SenderState {
            state: self.all_by_sender.get(&sender).cloned(),
            sender,
        };

        let mut updates = Vec::<Update>::with_capacity(128);
        let result = self.add_from_gossip_exn_impl(
            cmd,
            current_nonce,
            balance,
            &mut by_sender,
            &mut updates,
        )?;

        self.set_sender(by_sender);
        self.apply_updates(updates);

        Ok(result)
    }

    fn add_from_gossip_exn_impl(
        &self,
        cmd: &ValidCommandWithHash,
        current_nonce: Nonce,
        balance: Balance,
        by_sender: &mut SenderState,
        updates: &mut Vec<Update>,
    ) -> Result<(ValidCommandWithHash, VecDeque<ValidCommandWithHash>), CommandError> {
        let IndexedPoolConfig { slot_tx_end, .. } = &self.config;

        let current_global_slot = self.current_global_slot();

        if !slot_tx_end
            .as_ref()
            .map(|slot_tx_end| current_global_slot < *slot_tx_end)
            .unwrap_or(true)
        {
            return Err(CommandError::AfterSlotTxEnd);
        }

        let unchecked = cmd.data.forget_check();
        let fee = unchecked.fee();
        let fee_per_wu = unchecked.fee_per_wu();
        let cmd_applicable_at_nonce = unchecked.applicable_at_nonce();

        let consumed = {
            self.check_expiry(&unchecked)?;
            let consumed = currency_consumed(&unchecked).ok_or(CommandError::Overflow)?;
            if !unchecked.fee_token().is_default() {
                return Err(CommandError::BadToken);
            }
            consumed
        };

        match by_sender.state.as_mut() {
            None => {
                if current_nonce != cmd_applicable_at_nonce {
                    return Err(CommandError::InvalidNonce {
                        account_nonce: current_nonce,
                        expected: cmd_applicable_at_nonce,
                    });
                }
                if consumed > balance.to_amount() {
                    return Err(CommandError::InsufficientFunds { balance, consumed });
                }

                let mut queue = Self::make_queue();
                queue.push_back(cmd.clone());
                by_sender.state = Some((queue, consumed));

                updates.push(Update::Add {
                    command: cmd.clone(),
                    fee_per_wu,
                    add_to_applicable_by_fee: true,
                });

                Ok((cmd.clone(), Self::make_queue()))
            }
            Some((queued_cmds, reserved_currency)) => {
                assert!(!queued_cmds.is_empty());
                let queue_applicable_at_nonce = {
                    let first = queued_cmds.front().unwrap();
                    first.data.forget_check().applicable_at_nonce()
                };
                let queue_target_nonce = {
                    let last = queued_cmds.back().unwrap();
                    last.data.forget_check().expected_target_nonce()
                };
                if queue_target_nonce == cmd_applicable_at_nonce {
                    let reserved_currency = consumed
                        .checked_add(reserved_currency)
                        .ok_or(CommandError::Overflow)?;

                    if !(reserved_currency <= balance.to_amount()) {
                        return Err(CommandError::InsufficientFunds {
                            balance,
                            consumed: reserved_currency,
                        });
                    }

                    queued_cmds.push_back(cmd.clone());

                    updates.push(Update::Add {
                        command: cmd.clone(),
                        fee_per_wu,
                        add_to_applicable_by_fee: false,
                    });

                    Ok((cmd.clone(), Self::make_queue()))
                } else if queue_applicable_at_nonce == current_nonce {
                    if !cmd_applicable_at_nonce
                        .between(&queue_applicable_at_nonce, &queue_target_nonce)
                    {
                        return Err(CommandError::InvalidNonce {
                            account_nonce: cmd_applicable_at_nonce,
                            expected: queue_applicable_at_nonce,
                        });
                    }

                    let replacement_index = queued_cmds
                        .iter()
                        .position(|cmd| {
                            let cmd_applicable_at_nonce_prime =
                                cmd.data.forget_check().applicable_at_nonce();
                            cmd_applicable_at_nonce <= cmd_applicable_at_nonce_prime
                        })
                        .unwrap();

                    let drop_queue = queued_cmds.split_off(replacement_index);

                    let to_drop = drop_queue.front().unwrap().data.forget_check();
                    assert!(cmd_applicable_at_nonce <= to_drop.applicable_at_nonce());

                    // We check the fee increase twice because we need to be sure the
                    // subtraction is safe.
                    {
                        let replace_fee = to_drop.fee();
                        if !(fee >= replace_fee) {
                            return Err(CommandError::InsufficientReplaceFee { replace_fee, fee });
                        }
                    }

                    let dropped = self.remove_with_dependents_exn_impl(
                        drop_queue.front().unwrap(),
                        by_sender,
                        updates,
                    );
                    assert_eq!(drop_queue, dropped);

                    let (cmd, _) = {
                        let (v, dropped) = self.add_from_gossip_exn_impl(
                            cmd,
                            current_nonce,
                            balance,
                            by_sender,
                            updates,
                        )?;
                        // We've already removed them, so this should always be empty.
                        assert!(dropped.is_empty());
                        (v, dropped)
                    };

                    let drop_head = dropped.front().cloned().unwrap();
                    let mut drop_tail = dropped.into_iter().skip(1).peekable();

                    let mut increment = fee.checked_sub(&to_drop.fee()).unwrap();
                    let mut dropped = None::<VecDeque<_>>;
                    let mut current_nonce = current_nonce;
                    let mut this_updates = Vec::with_capacity(128);

                    while let Some(cmd) = drop_tail.peek() {
                        if dropped.is_some() {
                            let cmd_unchecked = cmd.data.forget_check();
                            let replace_fee = cmd_unchecked.fee();

                            increment = increment.checked_sub(&replace_fee).ok_or_else(|| {
                                CommandError::InsufficientReplaceFee {
                                    replace_fee,
                                    fee: increment,
                                }
                            })?;
                        } else {
                            current_nonce = current_nonce.succ();
                            let by_sender_pre = by_sender.clone();
                            this_updates.clear();

                            match self.add_from_gossip_exn_impl(
                                cmd,
                                current_nonce,
                                balance,
                                by_sender,
                                &mut this_updates,
                            ) {
                                Ok((_cmd, dropped)) => {
                                    assert!(dropped.is_empty());
                                    updates.append(&mut this_updates);
                                }
                                Err(_) => {
                                    *by_sender = by_sender_pre;
                                    dropped = Some(drop_tail.clone().skip(1).collect());
                                    continue; // Don't go to next
                                }
                            }
                        }
                        let _ = drop_tail.next();
                    }

                    if !(increment >= REPLACE_FEE) {
                        return Err(CommandError::InsufficientReplaceFee {
                            replace_fee: REPLACE_FEE,
                            fee: increment,
                        });
                    }

                    let mut dropped = dropped.unwrap_or_else(Self::make_queue);
                    dropped.push_front(drop_head);

                    Ok((cmd, dropped))
                } else {
                    // Invalid nonce or duplicate transaction got in- either way error
                    Err(CommandError::InvalidNonce {
                        account_nonce: cmd_applicable_at_nonce,
                        expected: queue_target_nonce,
                    })
                }
            }
        }
    }

    fn expired_by_global_slot(&self) -> Vec<ValidCommandWithHash> {
        let global_slot_since_genesis = self.global_slot_since_genesis();

        self.transactions_with_expiration
            .iter()
            .filter(|(slot, _cmd)| **slot < global_slot_since_genesis)
            .flat_map(|(_slot, cmd)| cmd.iter().cloned())
            .collect()
    }

    fn expired(&self) -> Vec<ValidCommandWithHash> {
        self.expired_by_global_slot()
    }

    fn remove_expired(&mut self) -> Vec<ValidCommandWithHash> {
        let mut dropped = Vec::with_capacity(128);
        for cmd in self.expired() {
            if self.member(&cmd) {
                let removed = self.remove_with_dependents_exn(&cmd);
                dropped.extend(removed);
            }
        }
        dropped
    }

    fn remove_lowest_fee(&mut self) -> VecDeque<ValidCommandWithHash> {
        let Some(set) = self.min_fee().and_then(|fee| self.all_by_fee.get(&fee)) else {
            return VecDeque::new();
        };

        // TODO: Should `self.all_by_fee` be a `BTreeSet` instead ?
        let bset: BTreeSet<_> = set.iter().collect();
        // TODO: Not sure if OCaml compare the same way than we do
        let min = bset.first().map(|min| (*min).clone()).unwrap();

        self.remove_with_dependents_exn(&min)
    }

    /// Drop commands from the end of the queue until the total currency consumed is
    /// <= the current balance.
    fn drop_until_sufficient_balance(
        mut queue: VecDeque<ValidCommandWithHash>,
        mut currency_reserved: Amount,
        current_balance: Amount,
    ) -> (
        VecDeque<ValidCommandWithHash>,
        Amount,
        VecDeque<ValidCommandWithHash>,
    ) {
        let mut dropped_so_far = VecDeque::with_capacity(queue.len());

        while currency_reserved > current_balance {
            let last = queue.pop_back().unwrap();
            let consumed = currency_consumed(&last.data.forget_check()).unwrap();
            dropped_so_far.push_back(last);
            currency_reserved = currency_reserved.checked_sub(&consumed).unwrap();
        }

        (queue, currency_reserved, dropped_so_far)
    }

    fn revalidate<F>(&mut self, kind: RevalidateKind, get_account: F) -> Vec<ValidCommandWithHash>
    where
        F: Fn(&AccountId) -> Account,
    {
        let requires_revalidation = |account_id: &AccountId| match kind {
            RevalidateKind::EntirePool => true,
            RevalidateKind::Subset(set) => set.contains(account_id),
        };

        let mut dropped = Vec::new();

        for (sender, (mut queue, mut currency_reserved)) in self.all_by_sender.clone() {
            if !requires_revalidation(&sender) {
                continue;
            }
            let account: Account = get_account(&sender);
            let current_balance = {
                let global_slot = self.global_slot_since_genesis();
                account.liquid_balance_at_slot(global_slot).to_amount()
            };
            let first_cmd = queue.front().unwrap();
            let first_nonce = first_cmd.data.forget_check().applicable_at_nonce();

            if !(account.has_permission_to_send() && account.has_permission_to_increment_nonce()) {
                let this_dropped = self.remove_with_dependents_exn(first_cmd);
                dropped.extend(this_dropped);
            } else if account.nonce < first_nonce {
                let this_dropped = self.remove_with_dependents_exn(first_cmd);
                dropped.extend(this_dropped);
            } else {
                // current_nonce >= first_nonce
                let first_applicable_nonce_index = queue.iter().position(|cmd| {
                    let nonce = cmd.data.forget_check().applicable_at_nonce();
                    nonce == account.nonce
                });

                let keep_queue = match first_applicable_nonce_index {
                    Some(index) => queue.split_off(index),
                    None => Default::default(),
                };
                let drop_queue = queue;

                for cmd in &drop_queue {
                    currency_reserved = currency_reserved
                        .checked_sub(&currency_consumed(&cmd.data.forget_check()).unwrap())
                        .unwrap();
                }

                let (keep_queue, currency_reserved, dropped_for_balance) =
                    Self::drop_until_sufficient_balance(
                        keep_queue,
                        currency_reserved,
                        current_balance,
                    );

                let to_drop: Vec<_> = drop_queue.into_iter().chain(dropped_for_balance).collect();

                let Some(head) = to_drop.first() else {
                    continue;
                };

                self.remove_applicable_exn(head);
                self.update_remove_all_by_fee_and_hash_and_expiration(to_drop.clone());

                match keep_queue.front().cloned() {
                    None => {
                        self.all_by_sender.remove(&sender);
                    }
                    Some(first_kept) => {
                        let first_kept_unchecked = first_kept.data.forget_check();
                        self.all_by_sender
                            .insert(sender, (keep_queue, currency_reserved));
                        Self::map_set_insert(
                            &mut self.applicable_by_fee,
                            first_kept_unchecked.fee_per_wu(),
                            first_kept,
                        );
                    }
                }

                dropped.extend(to_drop);
            }
        }

        dropped
    }
}

fn currency_consumed(cmd: &UserCommand) -> Option<Amount> {
    use crate::scan_state::transaction_logic::signed_command::{Body::*, PaymentPayload};

    let fee_amount = Amount::of_fee(&cmd.fee());
    let amount = match cmd {
        UserCommand::SignedCommand(c) => {
            match &c.payload.body {
                Payment(PaymentPayload { amount, .. }) => {
                    // The fee-payer is also the sender account, include the amount.
                    *amount
                }
                StakeDelegation(_) => Amount::zero(),
            }
        }
        UserCommand::ZkAppCommand(_) => Amount::zero(),
    };

    fee_amount.checked_add(&amount)
}

type BlakeHash = Arc<[u8; 32]>;

mod transaction_hash {
    use blake2::{
        digest::{Update, VariableOutput},
        Blake2bVar,
    };
    use mina_signer::Signature;

    use crate::scan_state::transaction_logic::{
        signed_command::SignedCommand, zkapp_command::AccountUpdate,
    };

    use super::*;

    pub fn hash_command(cmd: valid::UserCommand) -> ValidCommandWithHash {
        use mina_p2p_messages::binprot::BinProtWrite;

        fn to_binprot<T: Into<V>, V: BinProtWrite>(v: T) -> Vec<u8> {
            let value = v.into();
            let mut buffer = Vec::with_capacity(32 * 1024);
            value.binprot_write(&mut buffer).unwrap();
            buffer
        }

        let buffer: Vec<u8> = match &cmd {
            valid::UserCommand::SignedCommand(cmd) => {
                let mut cmd: SignedCommand = (**cmd).clone();
                cmd.signature = Signature::dummy();
                to_binprot::<_, v2::MinaBaseSignedCommandStableV2>(&cmd)
            }
            valid::UserCommand::ZkAppCommand(cmd) => {
                let mut cmd = cmd.clone().forget();
                cmd.fee_payer.authorization = Signature::dummy();
                cmd.account_updates = cmd.account_updates.map_to(|account_update| {
                    let dummy_auth = account_update.authorization.dummy();
                    AccountUpdate {
                        authorization: dummy_auth,
                        ..account_update.clone()
                    }
                });
                to_binprot::<_, v2::MinaBaseZkappCommandTStableV1WireStableV1>(&cmd)
            }
        };

        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        hasher.update(&buffer);

        let hash: Arc<[u8; 32]> = {
            let mut buffer = [0; 32];
            hasher
                .finalize_variable(&mut buffer)
                .expect("Invalid buffer size"); // Never occur
            Arc::from(buffer)
        };

        WithHash { data: cmd, hash }
    }
}

// TODO: Remove this
struct Envelope<T> {
    pub data: T,
}

impl<T> Envelope<T> {
    fn data(&self) -> &T {
        &self.data
    }

    fn is_sender_local(&self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub enum ApplyDecision {
    Accept,
    Reject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Time {
    nanoseconds_since_unix_epoch: u64,
}

impl Time {
    fn now() -> Self {
        const NANOS_PER_SECOND: u64 = 1000000000;

        let mut tp = libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        let result = unsafe {
            // Use same syscall than OCaml:
            // https://github.com/janestreet/time_now/blob/d7e3801d2f120b6723c28429de0dd63b669d47b8/src/time_now_stubs.c#L30
            libc::gettimeofday(&mut tp, std::ptr::null_mut())
        };
        if result == -1 {
            return Self {
                nanoseconds_since_unix_epoch: 0,
            };
        }

        Self {
            nanoseconds_since_unix_epoch: NANOS_PER_SECOND
                .wrapping_mul(tp.tv_sec as u64)
                .wrapping_add((tp.tv_usec as u64).wrapping_mul(1000)),
        }
    }
}

const MAX_PER_15_SECONDS: usize = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPool {
    pool: IndexedPool,
    locally_generated_uncommitted: HashMap<ValidCommandWithHash, (Time, Batch)>,
    locally_generated_committed: HashMap<ValidCommandWithHash, (Time, Batch)>,
    current_batch: usize,
    remaining_in_batch: usize,
    config: Config,
    batcher: (),
    best_tip_diff_relay: Option<()>,
    #[serde(skip)]
    best_tip_ledger: Option<Mask>,
    verification_key_table: VkRefcountTable,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pool: todo!(),
            locally_generated_uncommitted: todo!(),
            locally_generated_committed: todo!(),
            current_batch: todo!(),
            remaining_in_batch: todo!(),
            config: todo!(),
            batcher: todo!(),
            best_tip_diff_relay: todo!(),
            best_tip_ledger: todo!(),
            verification_key_table: todo!(),
        }
    }

    pub fn get_accounts_to_revalidate_on_new_best_tip(&self) -> BTreeSet<AccountId> {
        self.pool.all_by_sender.keys().cloned().collect()
    }

    pub fn on_new_best_tip(&mut self, accounts: &BTreeMap<AccountId, Account>) {
        // let validation_ledger = new_best_tip;
        // self.best_tip_ledger.replace(validation_ledger.clone());

        let dropped = self
            .pool
            .revalidate(RevalidateKind::EntirePool, |sender_id| {
                accounts
                    .get(sender_id)
                    .cloned()
                    .unwrap_or_else(Account::empty)
            });

        let dropped_locally_generated = dropped
            .iter()
            .filter(|cmd| {
                let dropped_commited = self.locally_generated_committed.remove(cmd).is_some();
                let dropped_uncommited = self.locally_generated_uncommitted.remove(cmd).is_some();
                // Nothing should be in both tables.
                assert!(!(dropped_commited && dropped_uncommited));
                dropped_commited || dropped_uncommited
            })
            .collect::<Vec<_>>();

        if !dropped_locally_generated.is_empty() {
            eprintln!(
                "Dropped locally generated commands $cmds from pool when transition frontier was recreated. {:?}",
                dropped_locally_generated
            )
        }
    }

    fn has_sufficient_fee(&self, pool_max_size: usize, cmd: &valid::UserCommand) -> bool {
        match self.pool.min_fee() {
            None => true,
            Some(min_fee) => {
                if self.pool.size() >= pool_max_size {
                    cmd.forget_check().fee_per_wu() > min_fee
                } else {
                    true
                }
            }
        }
    }

    fn drop_until_below_max_size(&mut self, pool_max_size: usize) -> Vec<ValidCommandWithHash> {
        let mut list = Vec::new();

        while self.pool.size() > pool_max_size {
            let dropped = self.pool.remove_lowest_fee();
            assert!(!dropped.is_empty());
            list.extend(dropped)
        }

        list
    }

    pub fn handle_transition_frontier_diff(
        &mut self,
        diff: &diff::BestTipDiff,
        best_tip_ledger: Mask,
    ) {
        let diff::BestTipDiff {
            new_commands,
            removed_commands,
            reorg_best_tip: _,
        } = diff;

        let global_slot = self.pool.global_slot_since_genesis();
        self.best_tip_ledger = Some(best_tip_ledger.clone());

        let pool_max_size = self.config.pool_max_size;

        self.verification_key_table.increment_list(new_commands);
        self.verification_key_table.decrement_list(removed_commands);

        let mut dropped_backtrack = Vec::with_capacity(256);
        for cmd in removed_commands {
            let cmd = transaction_hash::hash_command(cmd.data.clone());

            if let Some(time_added) = self.locally_generated_committed.remove(&cmd) {
                self.locally_generated_uncommitted
                    .insert(cmd.clone(), time_added);
            }

            let dropped_seq = match self.pool.add_from_backtrack(cmd) {
                Ok(_) => self.drop_until_below_max_size(pool_max_size),
                Err(_) => todo!(), // TODO: print error
            };
            dropped_backtrack.extend(dropped_seq);
        }

        self.verification_key_table
            .decrement_hashed(&dropped_backtrack);

        let locally_generated_dropped = dropped_backtrack
            .iter()
            .filter(|t| self.locally_generated_uncommitted.contains_key(t))
            .collect::<Vec<_>>();

        let dropped_commands = {
            let accounts_to_check = new_commands
                .iter()
                .chain(removed_commands)
                .flat_map(|cmd| cmd.data.forget_check().accounts_referenced())
                .collect::<HashSet<_>>();

            let existing_account_states_by_id =
                preload_accounts(&best_tip_ledger, &accounts_to_check);

            let get_account = |id: &AccountId| {
                match existing_account_states_by_id.get(id) {
                    Some(account) => account.clone(),
                    None => {
                        if accounts_to_check.contains(id) {
                            Account::empty()
                        } else {
                            // OCaml panic too, with same message
                            panic!(
                                "did not expect Indexed_pool.revalidate to call \
                                    get_account on account not in accounts_to_check"
                            )
                        }
                    }
                }
            };

            self.pool
                .revalidate(RevalidateKind::Subset(&accounts_to_check), get_account)
        };

        let (committed_commands, dropped_commit_conflicts): (Vec<_>, Vec<_>) = {
            let command_hashes: HashSet<BlakeHash> = new_commands
                .iter()
                .map(|cmd| {
                    let cmd = transaction_hash::hash_command(cmd.data.clone());
                    cmd.hash
                })
                .collect();

            dropped_commands
                .iter()
                .partition(|cmd| command_hashes.contains(&cmd.hash))
        };

        for cmd in &committed_commands {
            self.verification_key_table.decrement_hashed([&**cmd]);
            if let Some(data) = self.locally_generated_uncommitted.remove(cmd) {
                let old = self
                    .locally_generated_committed
                    .insert((*cmd).clone(), data);
                assert!(old.is_none());
            };
        }

        let _commit_conflicts_locally_generated = dropped_commit_conflicts
            .iter()
            .filter(|cmd| self.locally_generated_uncommitted.remove(cmd).is_some());

        for cmd in locally_generated_dropped {
            // If the dropped transaction was included in the winning chain, it'll
            // be in locally_generated_committed. If it wasn't, try re-adding to
            // the pool.
            let remove_cmd = |this: &mut Self| {
                this.verification_key_table.decrement_hashed([cmd]);
                assert!(this.locally_generated_uncommitted.remove(cmd).is_some());
            };

            if !self.locally_generated_committed.contains_key(cmd) {
                if !self.has_sufficient_fee(pool_max_size, &cmd.data) {
                    remove_cmd(self)
                } else {
                    let unchecked = &cmd.data;
                    match best_tip_ledger
                        .location_of_account(&unchecked.fee_payer())
                        .and_then(|addr| best_tip_ledger.get(addr))
                    {
                        Some(account) => {
                            match self.pool.add_from_gossip_exn(
                                cmd,
                                account.nonce,
                                account.liquid_balance_at_slot(global_slot),
                            ) {
                                Err(_) => {
                                    remove_cmd(self);
                                }
                                Ok(_) => {
                                    self.verification_key_table.increment_hashed([cmd]);
                                }
                            }
                        }
                        None => {
                            remove_cmd(self);
                        }
                    }
                }
            }
        }

        let expired_commands = self.pool.remove_expired();
        for cmd in &expired_commands {
            self.verification_key_table.decrement_hashed([cmd]);
            self.locally_generated_uncommitted.remove(cmd);
        }
    }

    fn apply(
        &mut self,
        diff: &diff::DiffVerified,
        is_sender_local: bool,
    ) -> Result<
        (
            ApplyDecision,
            Vec<ValidCommandWithHash>,
            Vec<(ValidCommandWithHash, diff::Error)>,
        ),
        String,
    > {
        let ledger = self.best_tip_ledger.as_ref().ok_or_else(|| {
            "Got transaction pool diff when transitin frontier is unavailable, ignoring."
                .to_string()
        })?;

        let fee_payer = |cmd: &ValidCommandWithHash| cmd.data.fee_payer();

        let fee_payer_account_ids: HashSet<_> = diff.list.iter().map(fee_payer).collect();
        let fee_payer_accounts = preload_accounts(ledger, &fee_payer_account_ids);

        let check_command = |pool: &IndexedPool, cmd: &ValidCommandWithHash| {
            if pool.member(cmd) {
                Err(diff::Error::Duplicate)
            } else {
                match fee_payer_accounts.get(&fee_payer(cmd)) {
                    None => Err(diff::Error::FeePayerAccountNotFound),
                    Some(account) => {
                        if account.has_permission_to_send()
                            && account.has_permission_to_increment_nonce()
                        {
                            Ok(())
                        } else {
                            Err(diff::Error::FeePayerNotPermittedToSend)
                        }
                    }
                }
            }
        };

        let add_results = diff
            .list
            .iter()
            .map(|cmd| {
                let result: Result<_, diff::Error> = (|| {
                    check_command(&self.pool, cmd)?;

                    let global_slot = self.pool.global_slot_since_genesis();
                    let account = fee_payer_accounts.get(&fee_payer(cmd)).unwrap(); // OCaml panics too

                    match self.pool.add_from_gossip_exn(
                        cmd,
                        account.nonce,
                        account.liquid_balance_at_slot(global_slot),
                    ) {
                        Ok(x) => Ok(x),
                        Err(e) => {
                            eprintln!();
                            Err(e.into())
                        }
                    }
                })();

                match result {
                    Ok((cmd, dropped)) => Ok((cmd, dropped)),
                    Err(err) => Err((cmd, err)),
                }
            })
            .collect::<Vec<_>>();

        let added_cmds = add_results
            .iter()
            .filter_map(|cmd| match cmd {
                Ok((cmd, _)) => Some(cmd),
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let dropped_for_add = add_results
            .iter()
            .filter_map(|cmd| match cmd {
                Ok((_, dropped)) => Some(dropped),
                Err(_) => None,
            })
            .flatten()
            .collect::<Vec<_>>();

        let dropped_for_size = { self.drop_until_below_max_size(self.config.pool_max_size) };

        let all_dropped_cmds = dropped_for_add
            .iter()
            .map(|cmd| *cmd)
            .chain(dropped_for_size.iter())
            .collect::<Vec<_>>();

        let _ = {
            self.verification_key_table.increment_hashed(added_cmds);
            self.verification_key_table
                .decrement_hashed(all_dropped_cmds.iter().map(|cmd| *cmd));
        };

        let dropped_for_add_hashes: HashSet<&BlakeHash> =
            dropped_for_add.iter().map(|cmd| &cmd.hash).collect();
        let dropped_for_size_hashes: HashSet<&BlakeHash> =
            dropped_for_size.iter().map(|cmd| &cmd.hash).collect();
        let all_dropped_cmd_hashes: HashSet<&BlakeHash> = dropped_for_add_hashes
            .union(&dropped_for_size_hashes)
            .map(|hash| *hash)
            .collect();

        // let locally_generated_dropped = all_dropped_cmds
        //     .iter()
        //     .filter(|cmd| self.locally_generated_uncommitted.remove(cmd).is_some());

        if is_sender_local {
            for result in add_results.iter() {
                let Ok((cmd, _dropped)) = result else {
                    continue;
                };
                if !all_dropped_cmd_hashes.contains(&cmd.hash) {
                    self.register_locally_generated(cmd);
                }
            }
        }

        let mut accepted = Vec::with_capacity(128);
        let mut rejected = Vec::with_capacity(128);

        // TODO: Re-work this to avoid cloning ?
        for result in &add_results {
            match result {
                Ok((cmd, _dropped)) => {
                    if all_dropped_cmd_hashes.contains(&cmd.hash) {
                        // ignored (dropped)
                    } else {
                        accepted.push(cmd.clone());
                    }
                }
                Err((cmd, error)) => {
                    rejected.push(((*cmd).clone(), error.clone()));
                }
            }
        }

        let decision = if rejected
            .iter()
            .any(|(_, error)| error.grounds_for_diff_rejection())
        {
            ApplyDecision::Reject
        } else {
            ApplyDecision::Accept
        };

        Ok((decision, accepted, rejected))
    }

    pub fn unsafe_apply(
        &mut self,
        diff: &diff::DiffVerified,
        is_sender_local: bool,
    ) -> Result<
        (
            ApplyDecision,
            Vec<UserCommand>,
            Vec<(UserCommand, diff::Error)>,
        ),
        String,
    > {
        let (decision, accepted, rejected) = self.apply(diff, is_sender_local)?;
        let accepted = accepted
            .into_iter()
            .map(|cmd| cmd.data.forget_check())
            .collect::<Vec<_>>();
        let rejected = rejected
            .into_iter()
            .map(|(cmd, e)| (cmd.data.forget_check(), e))
            .collect::<Vec<_>>();
        Ok((decision, accepted, rejected))
    }

    fn register_locally_generated(&mut self, cmd: &ValidCommandWithHash) {
        match self.locally_generated_uncommitted.entry(cmd.clone()) {
            Entry::Occupied(mut entry) => {
                let (time, _batch_num) = entry.get_mut();
                *time = Time::now();
            }
            Entry::Vacant(entry) => {
                let batch_num = if self.remaining_in_batch > 0 {
                    self.remaining_in_batch = self.remaining_in_batch - 1;
                    self.current_batch
                } else {
                    self.remaining_in_batch = MAX_PER_15_SECONDS - 1;
                    self.current_batch = self.current_batch + 1;
                    self.current_batch
                };
                entry.insert((Time::now(), Batch::Of(batch_num)));
            }
        }
    }

    fn verify(&self, diff: Envelope<diff::Diff>) -> Result<Vec<valid::UserCommand>, String> {
        let well_formedness_errors: HashSet<_> = diff
            .data()
            .list
            .iter()
            .flat_map(|cmd| match cmd.check_well_formedness() {
                Ok(()) => Vec::new(),
                Err(errors) => errors,
            })
            .collect();

        if !well_formedness_errors.is_empty() {
            return Err(format!(
                "Some commands have one or more well-formedness errors: {:?}",
                well_formedness_errors
            ));
        }

        let ledger = self.best_tip_ledger.as_ref().ok_or_else(|| {
            "We don't have a transition frontier at the moment, so we're unable to verify any transactions."
        })?;

        let cs = diff
            .data()
            .list
            .iter()
            .cloned()
            .map(|cmd| MaybeWithStatus { cmd, status: None })
            .collect::<Vec<_>>();

        let diff = UserCommand::to_all_verifiable::<FromUnappliedSequence, _>(cs, |account_ids| {
            let mempool_vks: HashMap<_, _> = account_ids
                .iter()
                .map(|id| {
                    let vks = self.verification_key_table.find_vks_by_account_id(id);
                    let vks: HashMap<_, _> =
                        vks.iter().map(|vk| (vk.hash, (*vk).clone())).collect();
                    (id.clone(), vks)
                })
                .collect();

            let ledger_vks = UserCommand::load_vks_from_ledger(account_ids, ledger);
            let ledger_vks: HashMap<_, _> = ledger_vks
                .into_iter()
                .map(|(id, vk)| {
                    let mut map = HashMap::new();
                    map.insert(vk.hash, vk);
                    (id, map)
                })
                .collect();

            let new_map: HashMap<AccountId, HashMap<Fp, VerificationKeyWire>> = HashMap::new();
            let merged =
                mempool_vks
                    .into_iter()
                    .chain(ledger_vks)
                    .fold(new_map, |mut accum, (id, map)| {
                        let entry = accum.entry(id).or_default();
                        for (hash, vk) in map {
                            entry.insert(hash, vk);
                        }
                        accum
                    });

            from_unapplied_sequence::Cache::new(merged)
        })
        .map_err(|e| format!("Invalid {:?}", e))?;

        let diff = diff
            .into_iter()
            .map(|MaybeWithStatus { cmd, status: _ }| WithStatus {
                data: cmd,
                status: Applied,
            })
            .collect::<Vec<_>>();

        Verifier
            .verify_commands(diff, None)
            .into_iter()
            .map(|cmd| {
                // TODO: Handle invalids
                match cmd {
                    crate::verifier::VerifyCommandsResult::Valid(cmd) => Ok(cmd),
                    e => Err(format!("invalid tx: {:?}", e)),
                }
            })
            .collect()
    }

    fn get_rebroadcastable<F>(&mut self, has_timed_out: F) -> Vec<Vec<UserCommand>>
    where
        F: Fn(&Time) -> bool,
    {
        let log = |has_timed_out: bool, s: &str, cmd: &ValidCommandWithHash| -> bool {
            if has_timed_out {
                eprintln!("{}: {:?}", s, cmd);
                false
            } else {
                true
            }
        };

        self.locally_generated_uncommitted
            .retain(|key, (time, _batch)| {
                log(
                    has_timed_out(time),
                    "No longer rebroadcasting uncommitted expired command",
                    key,
                )
            });
        self.locally_generated_committed
            .retain(|key, (time, _batch)| {
                log(
                    has_timed_out(time),
                    "Removing committed locally generated expired command",
                    key,
                )
            });

        let mut rebroadcastable_txs = self
            .locally_generated_uncommitted
            .iter()
            .collect::<Vec<_>>();

        rebroadcastable_txs.sort_by(|(txn1, (_, batch1)), (txn2, (_, batch2))| {
            use std::cmp::Ordering::Equal;

            let get_nonce =
                |txn: &ValidCommandWithHash| txn.data.forget_check().applicable_at_nonce();

            match batch1.cmp(batch2) {
                Equal => (),
                x => return x,
            }
            match get_nonce(txn1).cmp(&get_nonce(txn2)) {
                Equal => (),
                x => return x,
            }
            txn1.hash.cmp(&txn2.hash)
        });

        rebroadcastable_txs
            .into_iter()
            .group_by(|(_txn, (_time, batch))| batch)
            .into_iter()
            .map(|(_batch, group_txns)| {
                group_txns
                    .map(|(txn, _)| txn.data.forget_check())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Make sure that the merge in `TransactionPool::verify` is correct
    #[test]
    fn test_map_merge() {
        let mut a = HashMap::new();
        a.insert(1, {
            let mut map = HashMap::new();
            map.insert(1, 10);
            map.insert(2, 12);
            map
        });
        let mut b = HashMap::new();
        b.insert(1, {
            let mut map = HashMap::new();
            map.insert(3, 20);
            map
        });

        let new_map: HashMap<_, HashMap<_, _>> = HashMap::new();
        let merged = a
            .into_iter()
            .chain(b)
            .fold(new_map, |mut accum, (id, map)| {
                let entry = accum.entry(id).or_default();
                for (hash, vk) in map {
                    entry.insert(hash, vk);
                }
                accum
            });

        let one = merged.get(&1).unwrap();
        assert!(one.get(&1).is_some());
        assert!(one.get(&2).is_some());
        assert!(one.get(&3).is_some());

        dbg!(merged);
    }
}

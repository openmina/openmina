use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap, HashSet},
};

use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    scan_state::{
        currency::{Amount, Balance, Nonce, Slot},
        transaction_logic::{
            valid,
            zkapp_command::{
                from_unapplied_sequence::FromUnappliedSequence, MaybeWithStatus, WithHash,
            },
            UserCommand, WithStatus,
        },
    },
    Account, AccountId, BaseLedger, Mask, TokenId, VerificationKey,
};

type ValidCommandWithHash = WithHash<valid::UserCommand, BlakeHash>;

mod diff {
    use super::*;
    use crate::scan_state::transaction_logic::{valid, UserCommand, WithStatus};

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

    pub struct DiffVerified {
        pub list: Vec<ValidCommandWithHash>,
    }

    struct Rejected {
        list: Vec<(UserCommand, Error)>,
    }

    pub struct BestTipDiff {
        pub new_commands: Vec<WithStatus<valid::UserCommand>>,
        pub removed_commands: Vec<WithStatus<valid::UserCommand>>,
        pub reorg_best_tip: bool,
    }
}

fn preload_accounts(
    _ledger: &Mask,
    _account_ids: &HashSet<AccountId>,
) -> HashMap<AccountId, Account> {
    todo!()
    // let preload_accounts ledger account_ids =
    //   let existing_account_ids, existing_account_locs =
    //     Set.to_list account_ids
    //     |> Base_ledger.location_of_account_batch ledger
    //     |> List.filter_map ~f:(function
    //          | id, Some loc ->
    //              Some (id, loc)
    //          | _, None ->
    //              None )
    //     |> List.unzip
    //   in
    //   Base_ledger.get_batch ledger existing_account_locs
    //   |> List.map ~f:snd
    //   |> List.zip_exn existing_account_ids
    //   |> List.fold ~init:Account_id.Map.empty ~f:(fun map (id, maybe_account) ->
    //          let account =
    //            Option.value_exn maybe_account
    //              ~message:"Somehow a public key has a location but no account"
    //          in
    //          Map.add_exn map ~key:id ~data:account )
}

struct Config {
    trust_system: (),
    pool_max_size: usize,
    slot_tx_end: Option<Slot>,
}

pub type VerificationKeyWire = WithHash<VerificationKey>;

#[derive(Default)]
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

enum Batch {
    Of(usize),
}

pub enum CommandError {
    InvalidNonce {
        account_nonce: Nonce,
        expected: Nonce,
    },
    InsufficientFunds {
        balance: Amount,
        amount: Amount,
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
            CommandError::Overflow => diff::Error::Overflow,
            CommandError::BadToken => diff::Error::BadToken,
            CommandError::Expired { .. } => diff::Error::Expired,
            CommandError::UnwantedFeeToken { .. } => diff::Error::UnwantedFeeToken,
            CommandError::AfterSlotTxEnd => diff::Error::AfterSlotTxEnd,
        }
    }
}

struct IndexedPool {
    // global_slot_since_genesis: Slot,
}

enum RevalidateKind<'a> {
    EntirePool,
    Subset(&'a HashSet<AccountId>),
}

impl IndexedPool {
    fn size(&self) -> usize {
        todo!()
    }

    fn member(&self, cmd: &ValidCommandWithHash) -> bool {
        todo!()
    }

    fn global_slot_since_genesis(&self) -> Slot {
        todo!()
    }

    fn add_from_backtrack(&mut self, cmd: ValidCommandWithHash) -> Result<(), String> {
        todo!()
    }

    fn add_from_gossip_exn(
        &mut self,
        tx: &ValidCommandWithHash,
        nonce: Nonce,
        amount: Balance,
    ) -> Result<(ValidCommandWithHash, Vec<ValidCommandWithHash>), CommandError> {
        todo!()
    }

    fn remove_expired(&mut self) -> Vec<ValidCommandWithHash> {
        todo!()
    }

    fn drop_until_below_max_size(&mut self, pool_max_size: usize) -> Vec<ValidCommandWithHash> {
        todo!()
    }

    fn revalidate<F>(&mut self, kind: RevalidateKind, get_account: F) -> Vec<ValidCommandWithHash>
    where
        F: Fn(&AccountId) -> Account,
    {
        todo!()
    }
}

type BlakeHash = Box<[u8; 32]>;

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
        let hash: Box<[u8; 32]> = hasher.finalize_boxed().try_into().unwrap();

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
enum ApplyDecision {
    Accept,
    Reject,
}

const MAX_PER_15_SECONDS: usize = 10;

pub struct TransactionPool {
    pool: IndexedPool,
    locally_generated_uncommitted: HashMap<ValidCommandWithHash, (std::time::Instant, Batch)>,
    locally_generated_committed: HashMap<ValidCommandWithHash, (std::time::Instant, Batch)>,
    current_batch: usize,
    remaining_in_batch: usize,
    config: Config,
    batcher: (),
    best_tip_diff_relay: Option<()>,
    best_tip_ledger: Option<Mask>,
    verification_key_table: VkRefcountTable,
}

impl TransactionPool {
    fn has_sufficient_fee(&self, pool_max_size: usize, cmd: &valid::UserCommand) -> bool {
        todo!()
    }

    fn handle_transition_frontier_diff(&mut self, diff: diff::BestTipDiff, best_tip_ledger: Mask) {
        let diff::BestTipDiff {
            new_commands,
            removed_commands,
            reorg_best_tip,
        } = diff;

        let global_slot = self.pool.global_slot_since_genesis();
        self.best_tip_ledger = Some(best_tip_ledger.clone());

        let pool_max_size = self.config.pool_max_size;

        self.verification_key_table.increment_list(&new_commands);
        self.verification_key_table
            .decrement_list(&removed_commands);

        let mut dropped_backtrack = Vec::with_capacity(256);
        for cmd in &removed_commands {
            let cmd = transaction_hash::hash_command(cmd.data.clone());

            if let Some(time_added) = self.locally_generated_committed.remove(&cmd) {
                self.locally_generated_uncommitted
                    .insert(cmd.clone(), time_added);
            }

            let dropped_seq = match self.pool.add_from_backtrack(cmd) {
                Ok(_) => self.pool.drop_until_below_max_size(pool_max_size),
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
                .chain(&removed_commands)
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

        let commit_conflicts_locally_generated = dropped_commit_conflicts
            .iter()
            .filter(|cmd| self.locally_generated_uncommitted.remove(cmd).is_some())
            .collect::<Vec<_>>();

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
        diff: Envelope<diff::DiffVerified>,
    ) -> Result<
        (
            ApplyDecision,
            Vec<ValidCommandWithHash>,
            Vec<(ValidCommandWithHash, diff::Error)>,
        ),
        String,
    > {
        let is_sender_local = diff.is_sender_local();

        let ledger = self.best_tip_ledger.as_ref().ok_or_else(|| {
            "Got transaction pool diff when transitin frontier is unavailable, ignoring."
                .to_string()
        })?;

        let fee_payer = |cmd: &ValidCommandWithHash| cmd.data.fee_payer();

        let fee_payer_account_ids: HashSet<_> = diff.data().list.iter().map(fee_payer).collect();
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
            .data()
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

        let dropped_for_size = {
            self.pool
                .drop_until_below_max_size(self.config.pool_max_size)
        };

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

    fn register_locally_generated(&mut self, cmd: &ValidCommandWithHash) {
        match self.locally_generated_uncommitted.entry(cmd.clone()) {
            Entry::Occupied(mut entry) => {
                let (time, _batch_num) = entry.get_mut();
                *time = std::time::Instant::now();
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
                entry.insert((std::time::Instant::now(), Batch::Of(batch_num)));
            }
        }
    }

    fn verify(&self, diff: Envelope<diff::Diff>) -> Result<(), String> {
        let is_sender_local = diff.is_sender_local();

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

        // use crate::scan_state::transaction_logic::zkapp_command::last::Last;
        let cs = diff
            .data()
            .list
            .iter()
            .cloned()
            .map(|cmd| MaybeWithStatus { cmd, status: None })
            .collect::<Vec<_>>();
        UserCommand::to_all_verifiable::<FromUnappliedSequence, _>(cs, |account_ids| {
            todo!()
            // find_vk_via_ledger(ledger.clone(), expected_vk_hash, account_id)
        })
        .unwrap(); // TODO: No unwrap

        Ok(())
    }
}

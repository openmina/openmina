///
/// Pending_coinbase is to keep track of all the coinbase transactions that have been
/// applied to the ledger but for which there is no ledger proof yet. Every ledger
/// proof corresponds to a sequence of coinbase transactions which is part of all the
/// transactions it proves. Each of these sequences[Stack] are stored using the merkle
/// tree representation. The stacks are operated in a FIFO manner by keeping track of
/// its positions in the merkle tree. Whenever a ledger proof is emitted, the oldest
/// stack is removed from the tree and when a new coinbase is applied, the latest stack
/// is updated with the new coinbase.
///
/// The operations on the merkle tree of coinbase stacks include:
/// 1) adding a new singleton stack
/// 2) updating the latest stack when a new coinbase is added to it
/// 2) deleting the oldest stack
///
/// A stack can be either be created or modified by pushing a coinbase on to it.
///
/// This module also provides an interface for the checked computations required required to prove it in snark
///
/// Stack operations are done for transaction snarks and tree operations are done for the blockchain snark*)
use std::{collections::HashMap, fmt::Write, marker::PhantomData};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_p2p_messages::v2;
use mina_signer::CompressedPubKey;
use sha2::{Digest, Sha256};

use crate::{
    hash_noinputs, hash_with_kimchi,
    proofs::{
        field::{field, Boolean},
        numbers::{
            currency::{CheckedAmount, CheckedCurrency},
            nat::{CheckedNat, CheckedSlot},
        },
        transaction::transaction_snark::{checked_hash, CONSTRAINT_CONSTANTS},
        witness::Witness,
    },
    staged_ledger::hash::PendingCoinbaseAux,
    Address, Inputs, MerklePath, ToInputs,
};

use self::merkle_tree::MiniMerkleTree;

use super::{
    currency::{Amount, Magnitude, Slot},
    transaction_logic::Coinbase,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackId(u64);

impl std::fmt::Debug for StackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("StackId({})", self.0))
    }
}

impl StackId {
    pub fn incr_by_one(&self) -> Self {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| "Stack_id overflow".to_string())
            .unwrap()
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub(super) fn new(number: u64) -> Self {
        Self(number)
    }

    pub(super) fn as_u64(&self) -> u64 {
        self.0
    }
}

struct CoinbaseData {
    receiver: CompressedPubKey,
    amount: Amount,
}

impl ToInputs for CoinbaseData {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self { receiver, amount } = self;
        inputs.append(receiver);
        inputs.append(amount);
    }
}

impl CoinbaseData {
    pub fn empty() -> Self {
        Self {
            receiver: CompressedPubKey::empty(),
            amount: Amount::zero(),
        }
    }

    pub fn of_coinbase(cb: Coinbase) -> Self {
        let Coinbase {
            receiver,
            amount,
            fee_transfer: _,
        } = cb;
        Self { receiver, amount }
    }

    pub fn genesis() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug)]
pub struct StackState {
    pub source: Stack,
    pub target: Stack,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CoinbaseStack(pub Fp);

impl ToInputs for CoinbaseStack {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self(fp) = self;

        inputs.append(fp)
    }
}

impl std::fmt::Debug for CoinbaseStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            f.write_fmt(format_args!("CoinbaseStack(Empty)"))
        } else {
            f.debug_tuple("CoinbaseStack").field(&self.0).finish()
        }
    }
}

impl CoinbaseStack {
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/pending_coinbase.ml#L180
    pub fn push(&self, cb: Coinbase) -> Self {
        let mut inputs = Inputs::new();

        inputs.append(&CoinbaseData::of_coinbase(cb));
        inputs.append_field(self.0);

        let hash = hash_with_kimchi("CoinbaseStack", &inputs.to_fields());
        Self(hash)
    }

    pub fn checked_push(&self, cb: Coinbase, w: &mut Witness<Fp>) -> Self {
        let mut inputs = Inputs::new();

        inputs.append(&CoinbaseData::of_coinbase(cb));
        inputs.append_field(self.0);

        let hash = checked_hash("CoinbaseStack", &inputs.to_fields(), w);
        Self(hash)
    }

    fn check_merge(
        (_, t1): (&Self, &Self),
        (s2, _): (&Self, &Self),
        w: &mut Witness<Fp>,
    ) -> Boolean {
        field::equal(t1.0, s2.0, w)
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/pending_coinbase.ml#L188
    pub fn empty() -> Self {
        Self(hash_noinputs("CoinbaseStack"))
    }

    /// Used for tests/debug only
    fn is_empty(&self) -> bool {
        self == &Self::empty()
    }
}

type StackHash = Fp;

#[derive(Clone, PartialEq, Eq)]
pub struct StateStack {
    pub init: StackHash,
    pub curr: StackHash,
}

impl ToInputs for StateStack {
    /// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/mina_base/pending_coinbase.ml#L271
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self { init, curr } = self;

        inputs.append(init);
        inputs.append(curr);
    }
}

impl std::fmt::Debug for StateStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            f.write_fmt(format_args!("StateStack(Empty)"))
        } else {
            f.debug_struct("StateStack")
                .field("init", &self.init)
                .field("curr", &self.curr)
                .finish()
        }
    }
}

impl StateStack {
    fn push(&self, state_body_hash: Fp, global_slot: Slot) -> Self {
        let mut inputs = Inputs::new();

        inputs.append_field(self.curr);
        inputs.append_field(state_body_hash);
        inputs.append_field(global_slot.to_field());

        let hash = hash_with_kimchi("MinaProtoState", &inputs.to_fields());

        Self {
            init: self.init,
            curr: hash,
        }
    }

    fn checked_push(
        &self,
        state_body_hash: Fp,
        global_slot: CheckedSlot<Fp>,
        w: &mut Witness<Fp>,
    ) -> Self {
        let mut inputs = Inputs::new();

        inputs.append_field(self.curr);
        inputs.append_field(state_body_hash);
        inputs.append_field(global_slot.to_field());

        let hash = checked_hash("MinaProtoState", &inputs.to_fields(), w);

        Self {
            init: self.init,
            curr: hash,
        }
    }

    fn equal_var(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
        let b1 = field::equal(self.init, other.init, w);
        let b2 = field::equal(self.curr, other.curr, w);
        b1.and(&b2, w)
    }

    fn check_merge(
        (s1, t1): (&Self, &Self),
        (s2, t2): (&Self, &Self),
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let eq_src = s1.equal_var(s2, w);
        let eq_target = t1.equal_var(t2, w);
        let correct_transition = t1.equal_var(s2, w);
        let same_update = eq_src.and(&eq_target, w);
        Boolean::any(&[same_update, correct_transition], w)
    }

    fn empty() -> Self {
        Self {
            init: Fp::zero(),
            curr: Fp::zero(),
        }
    }

    /// Used for tests/debug only
    fn is_empty(&self) -> bool {
        self.curr.is_zero() && self.init.is_zero()
    }

    fn create(init: StackHash) -> Self {
        Self { init, curr: init }
    }
}

pub mod update {
    use crate::scan_state::currency::{Amount, Magnitude};

    #[derive(Debug)]
    pub enum Action {
        None,
        One,
        TwoCoinbaseInFirst,
        TwoCoinbaseInSecond,
    }

    #[derive(Debug)]
    pub enum StackUpdate {
        None,
        One(super::Stack),
        Two((super::Stack, super::Stack)),
    }

    #[derive(Debug)]
    pub struct Update {
        pub action: Action,
        pub coinbase_amount: Amount,
    }

    impl Update {
        fn genesis() -> Self {
            Self {
                action: Action::None,
                coinbase_amount: Amount::zero(),
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Stack {
    pub data: CoinbaseStack,
    pub state: StateStack,
}

impl ToInputs for Stack {
    /// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/mina_base/pending_coinbase.ml#L591
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self { data, state } = self;

        inputs.append(data);
        inputs.append(state);
    }
}

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() && self.state.is_empty() {
            f.write_fmt(format_args!("Stack(Empty)"))
        } else {
            f.debug_struct("Stack")
                .field("data", &self.data)
                .field("state", &self.state)
                .finish()
        }
    }
}

impl Stack {
    pub fn empty() -> Self {
        Self {
            data: CoinbaseStack::empty(),
            state: StateStack::empty(),
        }
    }

    pub fn push_coinbase(&self, cb: Coinbase) -> Self {
        Self {
            data: self.data.push(cb),
            state: self.state.clone(),
        }
    }

    pub fn push_state(&self, state_body_hash: Fp, global_slot: Slot) -> Self {
        Self {
            data: self.data.clone(),
            state: self.state.push(state_body_hash, global_slot),
        }
    }

    pub fn checked_push_coinbase(&self, cb: Coinbase, w: &mut Witness<Fp>) -> Self {
        Self {
            data: self.data.checked_push(cb, w),
            state: self.state.clone(),
        }
    }

    pub fn checked_push_state(
        &self,
        state_body_hash: Fp,
        global_slot: CheckedSlot<Fp>,
        w: &mut Witness<Fp>,
    ) -> Self {
        Self {
            data: self.data.clone(),
            state: self.state.checked_push(state_body_hash, global_slot, w),
        }
    }

    pub fn equal_var(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
        let b1 = field::equal(self.data.0, other.data.0, w);
        let b2 = {
            let b1 = field::equal(self.state.init, other.state.init, w);
            let b2 = field::equal(self.state.curr, other.state.curr, w);
            b1.and(&b2, w)
        };
        b1.and(&b2, w)
    }

    pub fn check_merge(
        transition1: (&Self, &Self),
        transition2: (&Self, &Self),
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let (s, t) = transition1;
        let (s2, t2) = transition2;

        let valid_coinbase_stacks =
            CoinbaseStack::check_merge((&s.data, &t.data), (&s2.data, &t2.data), w);
        let valid_state_stacks =
            StateStack::check_merge((&s.state, &t.state), (&s2.state, &t2.state), w);

        valid_coinbase_stacks.and(&valid_state_stacks, w)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/pending_coinbase.ml#L651
    pub fn create_with(other: &Self) -> Self {
        Self {
            state: StateStack::create(other.state.curr),
            ..Self::empty()
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/f5b013880dede0e2ef04cebf4b0213b850a85548/src/lib/mina_base/pending_coinbase.ml#L738
    pub fn var_create_with(other: &Self) -> Self {
        // Note: Here we use `init`
        Self {
            state: StateStack::create(other.state.init),
            ..Self::empty()
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/pending_coinbase.ml#L658
    pub fn connected(first: &Self, second: &Self, prev: Option<&Self>) -> bool {
        // same as old stack or second could be a new stack with empty data
        let coinbase_stack_connected =
            (first.data == second.data) || { second.data == CoinbaseStack::empty() };

        // 1. same as old stack or
        // 2. new stack initialized with the stack state of last block. Not possible to know this unless we track
        //    all the stack states because they are updated once per block (init=curr)
        // 3. [second] could be a new stack initialized with the latest state of [first] or
        // 4. [second] starts from the previous state of [first]. This is not available in either [first] or [second] *)
        let state_stack_connected = first.state == second.state
            || second.state.init == second.state.curr
            || first.state.curr == second.state.curr
            || prev
                .map(|prev| prev.state.curr == second.state.curr)
                .unwrap_or(true);

        coinbase_stack_connected && state_stack_connected
    }

    fn hash_var(&self, w: &mut Witness<Fp>) -> Fp {
        checked_hash("CoinbaseStack", &self.to_inputs_owned().to_fields(), w)
    }
}

#[derive(Clone, Debug)]
pub struct PendingCoinbase {
    pub(super) tree: merkle_tree::MiniMerkleTree<StackId, Stack, StackHasher>,
    pub(super) pos_list: Vec<StackId>,
    pub(super) new_pos: StackId,
}

#[derive(Clone, Debug)]
pub(super) struct StackHasher;

impl merkle_tree::TreeHasher<Stack> for StackHasher {
    fn hash_value(value: &Stack) -> Fp {
        let mut inputs = Inputs::new();

        inputs.append_field(value.data.0);

        inputs.append_field(value.state.init);
        inputs.append_field(value.state.curr);

        hash_with_kimchi("CoinbaseStack", &inputs.to_fields())
    }

    fn merge_hash(depth: usize, left: Fp, right: Fp) -> Fp {
        let param = format!("MinaCbMklTree{:03}", depth);

        crate::hash::hash_with_kimchi(param.as_str(), &[left, right])
    }

    fn empty_value() -> Stack {
        Stack::empty()
    }
}

impl PendingCoinbase {
    pub fn create(depth: usize) -> Self {
        let mut tree = MiniMerkleTree::create(depth);

        let nstacks = 2u64.pow(depth as u32);
        let mut stack_id = StackId::zero();

        assert_eq!(depth, 5);

        tree.fill_with((0..nstacks).map(|_| {
            let this_id = stack_id;
            stack_id = stack_id.incr_by_one();
            (this_id, Stack::empty())
        }));

        Self {
            tree,
            pos_list: Vec::with_capacity(128),
            new_pos: StackId::zero(),
        }
    }

    pub fn merkle_root(&mut self) -> Fp {
        self.tree.merkle_root()
    }

    fn get_stack(&self, addr: Address) -> &Stack {
        self.tree.get_exn(addr)
    }

    fn path(&mut self, addr: Address) -> Vec<MerklePath> {
        self.tree.path_exn(addr)
    }

    fn find_index(&self, key: StackId) -> Address {
        self.tree.find_index_exn(key)
    }

    fn next_index(&self, depth: usize) -> StackId {
        if self.new_pos.0 == (2u64.pow(depth as u32) - 1) {
            StackId::zero()
        } else {
            self.new_pos.incr_by_one()
        }
    }

    fn next_stack_id(&self, depth: usize, is_new_stack: bool) -> StackId {
        if is_new_stack {
            self.next_index(depth)
        } else {
            self.new_pos
        }
    }

    fn incr_index(&mut self, depth: usize, is_new_stack: bool) {
        if is_new_stack {
            let new_pos = self.next_index(depth);
            self.pos_list.push(self.new_pos);
            self.new_pos = new_pos;
        }
    }

    fn set_stack(&mut self, depth: usize, addr: Address, stack: Stack, is_new_stack: bool) {
        self.tree.set_exn(addr, stack);
        self.incr_index(depth, is_new_stack);
    }

    fn latest_stack_id(&self, is_new_stack: bool) -> StackId {
        if is_new_stack {
            self.new_pos
        } else {
            self.pos_list.last().cloned().unwrap_or_else(StackId::zero)
        }
    }

    fn current_stack_id(&self) -> Option<StackId> {
        self.pos_list.last().cloned()
    }

    fn current_stack(&self) -> &Stack {
        let prev_stack_id = self.current_stack_id().unwrap_or_else(StackId::zero);
        let addr = self.tree.find_index_exn(prev_stack_id);
        self.tree.get_exn(addr)
    }

    pub fn latest_stack(&self, is_new_stack: bool) -> Stack {
        let key = self.latest_stack_id(is_new_stack);
        let addr = self.tree.find_index_exn(key);
        let mut res = self.tree.get_exn(addr).clone();
        if is_new_stack {
            let prev_stack = self.current_stack();
            res.state = StateStack::create(prev_stack.state.curr);
        }
        res
    }

    fn oldest_stack_id(&self) -> Option<StackId> {
        self.pos_list.first().cloned()
    }

    fn remove_oldest_stack_id(&mut self) {
        todo!()
    }

    fn oldest_stack(&self) -> &Stack {
        let key = self.oldest_stack_id().unwrap_or_else(StackId::zero);
        let addr = self.find_index(key);
        self.get_stack(addr)
    }

    fn update_stack<F>(&mut self, depth: usize, is_new_stack: bool, fun: F)
    where
        F: FnOnce(&Stack) -> Stack,
    {
        let key = self.latest_stack_id(is_new_stack);
        let stack_addr = self.find_index(key);
        let stack_before = self.get_stack(stack_addr.clone());
        let stack_after = fun(stack_before);
        // state hash in "after" stack becomes previous state hash at top level
        self.set_stack(depth, stack_addr, stack_after, is_new_stack);
    }

    fn add_coinbase(&mut self, depth: usize, coinbase: Coinbase, is_new_stack: bool) {
        self.update_stack(depth, is_new_stack, |stack| stack.push_coinbase(coinbase))
    }

    fn add_state(
        &mut self,
        depth: usize,
        state_body_hash: Fp,
        global_slot: Slot,
        is_new_stack: bool,
    ) {
        self.update_stack(depth, is_new_stack, |stack| {
            stack.push_state(state_body_hash, global_slot)
        })
    }

    pub fn update_coinbase_stack(
        &mut self,
        depth: usize,
        stack: Stack,
        is_new_stack: bool,
    ) -> Result<(), String> {
        self.update_stack(depth, is_new_stack, |_| stack);
        Ok(())
    }

    pub fn remove_coinbase_stack(&mut self, depth: usize) -> Result<Stack, String> {
        let oldest_stack_id = if !self.pos_list.is_empty() {
            self.pos_list.remove(0) // TODO: Use `VecDeque`
        } else {
            return Err("No coinbase stack-with-state-hash to pop".to_string());
        };
        let stack_addr = self.find_index(oldest_stack_id);
        let stack = self.get_stack(stack_addr.clone()).clone();
        self.set_stack(depth, stack_addr, Stack::empty(), false);
        Ok(stack)
    }

    pub fn hash_extra(&self) -> PendingCoinbaseAux {
        let mut s = String::with_capacity(64 * 1024);
        for pos in self.pos_list.iter().rev() {
            write!(&mut s, "{}", pos.0).unwrap();
        }

        let mut sha = Sha256::new();
        sha.update(s.as_bytes());

        s.clear();
        write!(&mut s, "{}", self.new_pos.0).unwrap();
        sha.update(s);

        let digest = sha.finalize();
        PendingCoinbaseAux(digest.into())
    }

    pub fn pop_coinbases(
        proof_emitted: Boolean,
        pending_coinbase_witness: &mut PendingCoinbaseWitness,
        w: &mut Witness<Fp>,
    ) -> (Fp, Stack) {
        let addr = w.exists(pending_coinbase_witness.find_index_of_oldest_stack());
        let (prev, prev_path) = w.exists(pending_coinbase_witness.get_coinbase_stack(addr.clone()));

        checked_verify_merkle_path(&prev, &prev_path, w);

        let next = w.exists_no_check(match proof_emitted {
            Boolean::True => Stack::empty(),
            Boolean::False => prev.clone(),
        });

        pending_coinbase_witness.set_oldest_coinbase_stack(addr, next.clone());

        // Note: in OCaml hashing of `next` is made before `set_oldest_coinbase_stack`
        let new_root = checked_verify_merkle_path(&next, &prev_path, w);

        (new_root, prev)
    }

    pub fn add_coinbase_checked(
        update: &v2::MinaBasePendingCoinbaseUpdateStableV1,
        coinbase_receiver: &CompressedPubKey,
        supercharge_coinbase: Boolean,
        state_body_hash: Fp,
        global_slot: &CheckedSlot<Fp>,
        pending_coinbase_witness: &mut PendingCoinbaseWitness,
        w: &mut Witness<Fp>,
    ) -> Fp {
        let no_update = |[b0, b1]: &[Boolean; 2], w: &mut Witness<Fp>| b0.neg().and(&b1.neg(), w);
        let update_two_stacks_coinbase_in_first =
            |[b0, b1]: &[Boolean; 2], w: &mut Witness<Fp>| b0.neg().and(b1, w);
        let update_two_stacks_coinbase_in_second =
            |[b0, b1]: &[Boolean; 2], w: &mut Witness<Fp>| b0.and(b1, w);

        let v2::MinaBasePendingCoinbaseUpdateStableV1 {
            action,
            coinbase_amount: amount,
        } = update;

        let amount = Amount::from_u64(amount.as_u64()).to_checked();
        let (addr1, addr2) = w.exists(pending_coinbase_witness.find_index_of_newest_stacks());

        let action = {
            use v2::MinaBasePendingCoinbaseUpdateActionStableV1::*;
            match action {
                UpdateNone => [Boolean::False, Boolean::False],
                UpdateOne => [Boolean::True, Boolean::False],
                UpdateTwoCoinbaseInFirst => [Boolean::False, Boolean::True],
                UpdateTwoCoinbaseInSecond => [Boolean::True, Boolean::True],
            }
        };

        let no_update = no_update(&action, w);

        let update_state_stack = |stack: Stack,
                                  pending_coinbase_witness: &mut PendingCoinbaseWitness,
                                  w: &mut Witness<Fp>| {
            let previous_state_stack = w.exists(pending_coinbase_witness.get_previous_stack());
            let stack_initialized = Stack {
                state: previous_state_stack,
                ..stack.clone()
            };
            let stack_with_state_hash =
                stack_initialized.checked_push_state(state_body_hash, global_slot.clone(), w);
            w.exists_no_check(match no_update {
                Boolean::True => stack,
                Boolean::False => stack_with_state_hash,
            })
        };

        let update_stack1 = |stack: Stack,
                             pending_coinbase_witness: &mut PendingCoinbaseWitness,
                             w: &mut Witness<Fp>| {
            let stack = update_state_stack(stack, pending_coinbase_witness, w);
            let total_coinbase_amount = {
                let coinbase_amount = CONSTRAINT_CONSTANTS.coinbase_amount.to_checked::<Fp>();
                let superchaged_coinbase = CONSTRAINT_CONSTANTS
                    .coinbase_amount
                    .scale(CONSTRAINT_CONSTANTS.supercharged_coinbase_factor)
                    .unwrap()
                    .to_checked::<Fp>();

                match supercharge_coinbase {
                    Boolean::True => superchaged_coinbase,
                    Boolean::False => coinbase_amount,
                }
            };

            let rem_amount = total_coinbase_amount.sub(&amount, w);
            let no_coinbase_in_this_stack = update_two_stacks_coinbase_in_second(&action, w);

            let amount1_equal_to_zero = amount.equal(&CheckedAmount::zero(), w);
            let amount2_equal_to_zero = rem_amount.equal(&CheckedAmount::zero(), w);

            no_update.equal(&amount1_equal_to_zero, w);

            let no_coinbase = no_update.or(&no_coinbase_in_this_stack, w);

            let stack_with_amount1 = stack.checked_push_coinbase(
                Coinbase {
                    receiver: coinbase_receiver.clone(),
                    amount: amount.to_inner(), // TODO: Overflow ?
                    fee_transfer: None,
                },
                w,
            );

            let stack_with_amount2 = stack_with_amount1.checked_push_coinbase(
                Coinbase {
                    receiver: coinbase_receiver.clone(),
                    amount: rem_amount.to_inner(), // TODO: Overflow ?
                    fee_transfer: None,
                },
                w,
            );

            let on_false = {
                w.exists_no_check(match amount2_equal_to_zero {
                    Boolean::True => stack_with_amount1,
                    Boolean::False => stack_with_amount2,
                })
            };

            w.exists_no_check(match no_coinbase {
                Boolean::True => stack,
                Boolean::False => on_false,
            })
        };

        let update_stack2 = |init_stack: Stack, stack0: Stack, w: &mut Witness<Fp>| {
            let add_coinbase = update_two_stacks_coinbase_in_second(&action, w);
            let update_state = {
                let update_second_stack = update_two_stacks_coinbase_in_first(&action, w);
                update_second_stack.or(&add_coinbase, w)
            };

            let stack = {
                let stack_with_state = Stack {
                    state: StateStack::create(init_stack.state.curr),
                    ..stack0.clone()
                }
                .checked_push_state(state_body_hash, global_slot.clone(), w);
                w.exists_no_check(match update_state {
                    Boolean::True => stack_with_state,
                    Boolean::False => stack0,
                })
            };

            let stack_with_coinbase = stack.checked_push_coinbase(
                Coinbase {
                    receiver: coinbase_receiver.clone(),
                    amount: amount.to_inner(), // TODO: Overflow ?
                    fee_transfer: None,
                },
                w,
            );

            w.exists_no_check(match add_coinbase {
                Boolean::True => stack_with_coinbase,
                Boolean::False => stack,
            })
        };

        let (_new_root, prev, _updated_stack1) = {
            let (stack, path) = w.exists_no_check({
                let pc = &mut pending_coinbase_witness.pending_coinbase;
                let stack = pc.get_stack(addr1.clone()).clone();
                let path = pc.path(addr1.clone());
                (stack, path)
            });
            checked_verify_merkle_path(&stack, &path, w);

            let next = update_stack1(stack.clone(), pending_coinbase_witness, w);

            pending_coinbase_witness.set_coinbase_stack(addr1, next.clone());
            let new_root = checked_verify_merkle_path(&next, &path, w);
            (new_root, stack, next)
        };

        let (root, _, _) = {
            let (stack, path) = w.exists_no_check({
                let pc = &mut pending_coinbase_witness.pending_coinbase;
                let stack = pc.get_stack(addr2.clone()).clone();
                let path = pc.path(addr2.clone());
                (stack, path)
            });
            checked_verify_merkle_path(&stack, &path, w);

            let next = update_stack2(prev, stack.clone(), w);

            pending_coinbase_witness.set_coinbase_stack(addr2, next.clone());
            let new_root = checked_verify_merkle_path(&next, &path, w);
            (new_root, stack, next)
        };

        root
    }
}

/// `implied_root` in OCaml
pub fn checked_verify_merkle_path(
    account: &Stack,
    merkle_path: &[MerklePath],
    w: &mut Witness<Fp>,
) -> Fp {
    let account_hash = account.hash_var(w);
    let mut param = String::with_capacity(16);

    merkle_path
        .iter()
        .enumerate()
        .fold(account_hash, |accum, (depth, path)| {
            let hashes = match path {
                MerklePath::Left(right) => [accum, *right],
                MerklePath::Right(left) => [*left, accum],
            };

            param.clear();
            write!(&mut param, "MinaCbMklTree{:03}", depth).unwrap();

            w.exists(hashes);
            checked_hash(param.as_str(), &hashes, w)
        })
}

pub struct PendingCoinbaseWitness {
    pub pending_coinbase: PendingCoinbase,
    pub is_new_stack: bool,
}

impl PendingCoinbaseWitness {
    fn coinbase_stack_path_exn(&mut self, idx: Address) -> Vec<MerklePath> {
        self.pending_coinbase.path(idx)
    }

    fn find_index_of_oldest_stack(&self) -> Address {
        let stack_id = self
            .pending_coinbase
            .oldest_stack_id()
            .unwrap_or_else(StackId::zero);
        self.pending_coinbase.find_index(stack_id)
    }

    fn get_coinbase_stack(&mut self, idx: Address) -> (Stack, Vec<MerklePath>) {
        let elt = self.pending_coinbase.get_stack(idx.clone()).clone();
        let path = self.coinbase_stack_path_exn(idx);
        (elt, path)
    }

    fn set_oldest_coinbase_stack(&mut self, idx: Address, stack: Stack) {
        let depth = CONSTRAINT_CONSTANTS.pending_coinbase_depth as usize;
        self.pending_coinbase.set_stack(depth, idx, stack, false);
    }

    fn find_index_of_newest_stacks(&self) -> (Address, Address) {
        let depth = CONSTRAINT_CONSTANTS.pending_coinbase_depth as usize;

        let index1 = {
            let stack_id = self.pending_coinbase.latest_stack_id(self.is_new_stack);
            self.pending_coinbase.find_index(stack_id)
        };

        let index2 = {
            let stack_id = self
                .pending_coinbase
                .next_stack_id(depth, self.is_new_stack);
            self.pending_coinbase.find_index(stack_id)
        };

        (index1, index2)
    }

    fn get_previous_stack(&self) -> StateStack {
        if self.is_new_stack {
            let stack = self.pending_coinbase.current_stack();
            StateStack {
                init: stack.state.curr,
                curr: stack.state.curr,
            }
        } else {
            let stack = self.pending_coinbase.latest_stack(self.is_new_stack);
            stack.state
        }
    }

    fn set_coinbase_stack(&mut self, idx: Address, stack: Stack) {
        let depth = CONSTRAINT_CONSTANTS.pending_coinbase_depth as usize;
        self.pending_coinbase
            .set_stack(depth, idx, stack, self.is_new_stack);
    }
}

/// Keep it a bit generic, in case we need a merkle tree somewhere else
pub mod merkle_tree {
    use crate::{AccountIndex, Address, AddressIterator, Direction, HashesMatrix, MerklePath};

    use super::*;

    pub trait TreeHasher<V> {
        fn hash_value(value: &V) -> Fp;
        fn empty_value() -> V;
        fn merge_hash(depth: usize, left: Fp, right: Fp) -> Fp;
    }

    #[derive(Clone)]
    pub struct MiniMerkleTree<K, V, H> {
        pub values: Vec<V>,
        pub indexes: HashMap<K, Address>,
        pub hashes_matrix: HashesMatrix,
        pub depth: usize,
        pub _hasher: PhantomData<H>,
    }

    impl<K, V, H> std::fmt::Debug for MiniMerkleTree<K, V, H>
    where
        K: std::fmt::Debug + Ord,
        V: std::fmt::Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut indexes = self.indexes.iter().collect::<Vec<_>>();
            indexes.sort_by_key(|(key, _addr)| *key);
            // indexes.sort_by_key(|(_key, addr)| addr.to_index());

            f.debug_struct("MiniMerkleTree")
                .field("values", &self.values)
                .field("indexes_len", &indexes.len())
                .field("indexes", &indexes)
                // .field("hashes_matrix", &self.hashes_matrix)
                .field("depth", &self.depth)
                .finish()
        }
    }

    impl<K, V, H> MiniMerkleTree<K, V, H>
    where
        K: Eq + std::hash::Hash,
        H: TreeHasher<V>,
        V: PartialEq,
    {
        pub fn create(depth: usize) -> Self {
            let max_values = 2u64.pow(depth as u32) as usize;

            Self {
                values: Vec::with_capacity(max_values),
                indexes: HashMap::new(),
                depth,
                hashes_matrix: HashesMatrix::new(depth),
                _hasher: PhantomData,
            }
        }

        pub fn fill_with<I>(&mut self, data: I)
        where
            I: Iterator<Item = (K, V)>,
        {
            assert!(self.values.is_empty());

            assert_eq!(self.depth, 5);

            // OCaml uses those indexes
            let indexes: HashMap<usize, usize> = [
                (31, 31),
                (30, 15),
                (29, 23),
                (28, 7),
                (27, 27),
                (26, 11),
                (25, 19),
                (24, 3),
                (23, 29),
                (22, 13),
                (21, 21),
                (20, 5),
                (19, 25),
                (18, 9),
                (17, 17),
                (16, 1),
                (15, 30),
                (14, 14),
                (13, 22),
                (12, 6),
                (11, 26),
                (10, 10),
                (9, 18),
                (8, 2),
                (7, 28),
                (6, 12),
                (5, 20),
                (4, 4),
                (3, 24),
                (2, 8),
                (1, 16),
                (0, 0),
            ]
            .iter()
            .copied()
            .collect();

            for (index, (key, value)) in data.enumerate() {
                self.values.push(value);

                let index = indexes
                    .get(&index)
                    .copied()
                    .map(AccountIndex::from)
                    .unwrap();
                self.indexes
                    .insert(key, Address::from_index(index, self.depth));
            }
        }

        fn get(&self, addr: Address) -> Option<&V> {
            assert_eq!(addr.length(), self.depth);

            let index = addr.to_index().0 as usize;
            self.values.get(index)
        }

        pub fn get_exn(&self, addr: Address) -> &V {
            self.get(addr).unwrap()
        }

        pub fn set_exn(&mut self, addr: Address, value: V) {
            use std::cmp::Ordering::*;

            assert_eq!(addr.length(), self.depth);
            let index = addr.to_index().0 as usize;

            let mut invalidate = true;

            match index.cmp(&self.values.len()) {
                Less => {
                    invalidate = self.values[index] != value;
                    self.values[index] = value
                }
                Equal => self.values.push(value),
                Greater => panic!("wrong use of `set_exn`"),
            }

            if invalidate {
                self.hashes_matrix.invalidate_hashes(addr.to_index());
            }
        }

        pub fn find_index_exn(&self, key: K) -> Address {
            self.indexes.get(&key).cloned().unwrap()
        }

        pub fn path_exn(&mut self, addr: Address) -> Vec<MerklePath> {
            let mut merkle_path = Vec::with_capacity(addr.length());
            let mut path_to_addr = addr.into_iter();
            let root = Address::root();

            self.emulate_tree_to_get_path(root, &mut path_to_addr, &mut merkle_path);

            merkle_path
        }

        fn get_value_hash(&mut self, addr: Address) -> Fp {
            if let Some(hash) = self.hashes_matrix.get(&addr) {
                return *hash;
            }

            let hash = match self.get(addr.clone()) {
                Some(value) => H::hash_value(value),
                None => H::hash_value(&H::empty_value()),
            };

            self.hashes_matrix.set(&addr, hash);

            hash
        }

        fn get_node_hash(&mut self, addr: &Address, left: Fp, right: Fp) -> Fp {
            if let Some(hash) = self.hashes_matrix.get(addr) {
                return *hash;
            };

            let depth_in_tree = self.depth - addr.length();

            let hash = H::merge_hash(depth_in_tree - 1, left, right);
            self.hashes_matrix.set(addr, hash);
            hash
        }

        fn emulate_tree_to_get_path(
            &mut self,
            addr: Address,
            path: &mut AddressIterator,
            merkle_path: &mut Vec<MerklePath>,
        ) -> Fp {
            if addr.length() == self.depth {
                return self.get_value_hash(addr);
            }

            let next_direction = path.next();

            // We go until the end of the path
            if let Some(direction) = next_direction.as_ref() {
                let child = match direction {
                    Direction::Left => addr.child_left(),
                    Direction::Right => addr.child_right(),
                };
                self.emulate_tree_to_get_path(child, path, merkle_path);
            };

            let mut get_child_hash = |addr: Address| match self.hashes_matrix.get(&addr) {
                Some(hash) => *hash,
                None => {
                    if let Some(hash) = self.hashes_matrix.get(&addr) {
                        *hash
                    } else {
                        self.emulate_tree_to_get_path(addr, path, merkle_path)
                    }
                }
            };

            let left = get_child_hash(addr.child_left());
            let right = get_child_hash(addr.child_right());

            if let Some(direction) = next_direction {
                let hash = match direction {
                    Direction::Left => MerklePath::Left(right),
                    Direction::Right => MerklePath::Right(left),
                };
                merkle_path.push(hash);
            };

            self.get_node_hash(&addr, left, right)
        }

        pub fn merkle_root(&mut self) -> Fp {
            let root = Address::root();

            if let Some(hash) = self.hashes_matrix.get(&root) {
                return *hash;
            };

            self.emulate_tree_merkle_root(root)
        }

        pub fn emulate_tree_merkle_root(&mut self, addr: Address) -> Fp {
            let current_depth = self.depth - addr.length();

            if current_depth == 0 {
                return self.get_value_hash(addr);
            }

            let mut get_child_hash = |addr: Address| {
                if let Some(hash) = self.hashes_matrix.get(&addr) {
                    *hash
                } else {
                    self.emulate_tree_merkle_root(addr)
                }
            };

            let left_hash = get_child_hash(addr.child_left());
            let right_hash = get_child_hash(addr.child_right());

            self.get_node_hash(&addr, left_hash, right_hash)
        }
    }

    //   [%%define_locally
    //   M.
    //     ( of_hash
    //     , get_exn
    //     , path_exn
    //     , set_exn
    //     , find_index_exn
    //     , add_path
    //     , merkle_root )]
    // end
}

#[cfg(test)]
mod tests {
    use crate::FpExt;

    use super::{merkle_tree::MiniMerkleTree, *};

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_merkle_tree() {
        {
            const DEPTH: usize = 3;
            let mut tree = MiniMerkleTree::<StackId, Stack, StackHasher>::create(DEPTH);
            let merkle_root = tree.merkle_root();
            assert_eq!(
                merkle_root.to_decimal(),
                "9939061863620980199451530646711695641079091335264396436068661296746064363179"
            );
        }

        {
            const DEPTH: usize = 5;
            let mut tree = MiniMerkleTree::<StackId, Stack, StackHasher>::create(DEPTH);
            let merkle_root = tree.merkle_root();
            assert_eq!(
                merkle_root.to_decimal(),
                "25504365445533103805898245102289650498571312278321176071043666991586378788150"
            );
        }
    }
}

use ark_ff::Zero;
use mina_hasher::Fp;

use crate::{
    proofs::{
        numbers::{
            currency::{CheckedAmount, CheckedSigned},
            nat::{CheckedIndex, CheckedSlot},
        },
        to_field_elements::ToFieldElements,
        witness::{
            field, transaction_snark::checked_hash, Boolean, Check, FieldWitness, ToBoolean,
            Witness,
        },
        zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash},
    },
    scan_state::transaction_logic::{
        local_state::{StackFrame, StackFrameChecked, StackFrameCheckedFrame},
        zkapp_command::{AccountUpdate, CallForest, WithHash},
    },
    MyCow, TokenId,
};

use super::intefaces::{
    AmountInterface, CallForestInterface, CallStackInterface, GlobalSlotSinceGenesisInterface,
    GlobalStateInterface, IndexInterface, Opt, SignedAmountInterface, StackFrameInterface,
    StackInterface, WitnessGenerator,
};

use super::intefaces::WitnessGenerator as W;

impl<F: FieldWitness> WitnessGenerator<F> for Witness<F> {
    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        self.exists(data)
    }

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        self.exists_no_check(data)
    }
}

impl SignedAmountInterface for CheckedSigned<Fp, CheckedAmount<Fp>> {
    fn zero() -> Self {
        todo!()
    }
    fn is_neg(&self) -> Boolean {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn is_non_neg(&self) -> Boolean {
        todo!()
    }
    fn negate(&self) -> Self {
        todo!()
    }
    fn add_flagged(&self, other: &Self) -> (Self, Boolean) {
        todo!()
    }
    fn of_unsigned(fee: impl AmountInterface) -> Self {
        todo!()
    }
}

impl AmountInterface for CheckedAmount<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn add_flagged(&self, other: &Self) -> (Self, Boolean) {
        todo!()
    }
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Boolean) {
        todo!()
    }
    fn of_constant_fee(fee: crate::scan_state::currency::Fee) -> Self {
        todo!()
    }
}

impl CallForestInterface for WithHash<CallForest<AccountUpdate>> {
    type W = Witness<Fp>;

    fn empty() -> Self {
        WithHash {
            data: CallForest::empty(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w)
    }
    fn pop_exn(&self, w: &mut Self::W) -> ((AccountUpdate, Self), Self) {
        let Self { data, hash } = self;

        let hd_r = &data.first().unwrap().elt;
        let account_update = &hd_r.account_update;
        let auth = &account_update.authorization;

        w.exists(&account_update.body);

        // let pop_exn ({ hash = h; data = r } : t) : (account_update * t) * t =
        //   with_label "Zkapp_call_forest.pop_exn" (fun () ->
        //       let hd_r =
        //         V.create (fun () -> V.get r |> List.hd_exn |> With_stack_hash.elt)
        //       in
        //       let account_update = V.create (fun () -> (V.get hd_r).account_update) in
        //       let auth =
        //         V.(create (fun () -> (V.get account_update).authorization))
        //       in
        //       let account_update =
        //         exists (Account_update.Body.typ ()) ~compute:(fun () ->
        //             (V.get account_update).body )
        //       in
        //       let account_update =
        //         With_hash.of_data account_update
        //           ~hash_data:Zkapp_command.Digest.Account_update.Checked.create
        //       in
        //       let subforest : t =
        //         let subforest = V.create (fun () -> (V.get hd_r).calls) in
        //         let subforest_hash =
        //           exists Zkapp_command.Digest.Forest.typ ~compute:(fun () ->
        //               Zkapp_command.Call_forest.hash (V.get subforest) )
        //         in
        //         { hash = subforest_hash; data = subforest }
        //       in
        //       let tl_hash =
        //         exists Zkapp_command.Digest.Forest.typ ~compute:(fun () ->
        //             V.get r |> List.tl_exn |> Zkapp_command.Call_forest.hash )
        //       in
        //       let tree_hash =
        //         Zkapp_command.Digest.Tree.Checked.create
        //           ~account_update:account_update.hash ~calls:subforest.hash
        //       in
        //       let hash_cons =
        //         Zkapp_command.Digest.Forest.Checked.cons tree_hash tl_hash
        //       in
        //       F.Assert.equal hash_cons h ;
        //       ( ( ({ account_update; control = auth }, subforest)
        //         , { hash = tl_hash
        //           ; data = V.(create (fun () -> List.tl_exn (get r)))
        //           } )
        //         : (account_update * t) * t ) )

        todo!()
    }
}

impl StackFrameInterface for StackFrameChecked {
    type Calls = WithHash<CallForest<AccountUpdate>>;
    type W = Witness<Fp>;

    fn caller(&self) -> crate::TokenId {
        todo!()
    }
    fn caller_caller(&self) -> crate::TokenId {
        todo!()
    }
    fn calls(&self) -> &Self::Calls {
        &self.calls
    }
    fn make(caller: TokenId, caller_caller: TokenId, calls: &Self::Calls, w: &mut Self::W) -> Self {
        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls: calls.clone(),
        };
        Self::of_frame(frame)
    }
    fn on_if(self, w: &mut Self::W) -> Self {
        let frame: &StackFrameCheckedFrame = &*self;
        w.exists_no_check(frame);
        self
    }
}

fn call_stack_digest_checked_cons(h: Fp, t: Fp, w: &mut Witness<Fp>) -> Fp {
    checked_hash("MinaActUpStckFrmCons", &[h, t], w)
}

impl StackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type Elt = StackFrameChecked;
    type W = Witness<Fp>;

    fn empty() -> Self {
        WithHash {
            data: Vec::new(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w)
    }
    fn pop_exn(&self) -> (Self::Elt, Self) {
        todo!()
    }
    fn pop(&self, w: &mut Self::W) -> Opt<(Self::Elt, Self)> {
        let Self { data, hash } = self;
        let input_is_empty = self.is_empty(w);
        let hd_r = match data.first() {
            None => {
                let data = StackFrame::default();
                let hash = data.hash();
                MyCow::Own(WithHash { data, hash })
            }
            Some(x) => MyCow::Borrow(&x.elt),
        };
        let tl_r = data.get(1..).unwrap_or(&[]);
        let elt = hd_r.exists_elt(w);
        let stack = w.exists(match tl_r {
            [] => Fp::zero(),
            [x, ..] => x.stack_hash,
        });
        let stack_frame_hash = elt.hash(w);
        let h2 = call_stack_digest_checked_cons(stack_frame_hash, stack, w);
        let is_equal = field::equal(*hash, h2, w);
        Boolean::assert_any(&[input_is_empty, is_equal], w);
        Opt {
            is_some: input_is_empty.neg(),
            data: (
                elt,
                Self {
                    data: tl_r.to_vec(),
                    hash: stack,
                },
            ),
        }
    }
    fn push(&self, elt: Self::Elt) -> Self {
        todo!()
    }
}
impl CallStackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type StackFrame = StackFrameChecked;
}

impl GlobalStateInterface for GlobalStateForProof {
    type Ledger = LedgerWithHash;

    type SignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;

    fn first_pass_ledger(&self) -> Self::Ledger {
        self.first_pass_ledger.clone()
    }
    fn set_first_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn second_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn set_second_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn fee_excess(&self) -> Self::SignedAmount {
        todo!()
    }
    fn supply_increase(&self) -> Self::SignedAmount {
        todo!()
    }
}

impl IndexInterface for CheckedIndex<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn succ(&self) -> Self {
        todo!()
    }
}

impl GlobalSlotSinceGenesisInterface for CheckedSlot<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn greater_than(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
}

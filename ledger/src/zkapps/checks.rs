use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    checked_equal_compressed_key,
    proofs::{
        field::{field, Boolean, ToBoolean},
        numbers::common::ForZkappCheck,
        witness::Witness,
    },
    scan_state::transaction_logic::zkapp_command::{ClosedInterval, OrIgnore},
    MyCow,
};

/// Check zkapp preconditions
pub trait ZkappCheck {
    type T;

    fn zcheck<Ops: ZkappCheckOps>(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean;
}

impl<T> OrIgnore<T> {
    fn make_zcheck<Ops, CompareFn, DefaultFn>(
        &self,
        default_fn: CompareFn,
        compare_fun: DefaultFn,
        w: &mut Witness<Fp>,
    ) -> Boolean
    where
        Ops: ZkappCheckOps,
        CompareFn: Fn() -> T,
        DefaultFn: Fn(&T, &mut Witness<Fp>) -> Boolean,
    {
        let (is_some, value) = match self {
            OrIgnore::Check(v) => (Boolean::True, MyCow::Borrow(v)),
            OrIgnore::Ignore => (Boolean::False, MyCow::Own(default_fn())),
        };
        let is_good = compare_fun(value.as_ref(), w);
        Ops::boolean_any([is_some.neg(), is_good], w)
    }
}

impl<DefaultFn> ZkappCheck for (&OrIgnore<Boolean>, DefaultFn)
where
    DefaultFn: Fn() -> Boolean,
{
    type T = Boolean;

    fn zcheck<Ops: ZkappCheckOps>(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
        let (this, default_fn) = self;
        let compare = |value: &Self::T, w: &mut Witness<Fp>| Ops::is_boolean_equal(x, value, w);
        this.make_zcheck::<Ops, _, _>(default_fn, compare, w)
    }
}

impl<DefaultFn> ZkappCheck for (&OrIgnore<Fp>, DefaultFn)
where
    DefaultFn: Fn() -> Fp,
{
    type T = Fp;

    fn zcheck<Ops: ZkappCheckOps>(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
        let (this, default_fn) = self;
        let compare = |value: &Self::T, w: &mut Witness<Fp>| Ops::is_field_equal(x, value, w);
        this.make_zcheck::<Ops, _, _>(default_fn, compare, w)
    }
}

impl<DefaultFn> ZkappCheck for (&OrIgnore<CompressedPubKey>, DefaultFn)
where
    DefaultFn: Fn() -> CompressedPubKey,
{
    type T = CompressedPubKey;

    fn zcheck<Ops: ZkappCheckOps>(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
        let (this, default_fn) = self;
        let compare =
            |value: &Self::T, w: &mut Witness<Fp>| Ops::is_compressed_key_equal(x, value, w);
        this.make_zcheck::<Ops, _, _>(default_fn, compare, w)
    }
}

impl<T, DefaultFn> ZkappCheck for (&OrIgnore<ClosedInterval<T>>, DefaultFn)
where
    DefaultFn: Fn() -> ClosedInterval<T>,
    T: ForZkappCheck<Fp>,
{
    type T = T;

    fn zcheck<Ops: ZkappCheckOps>(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
        let (this, default_fn) = self;
        let compare = |value: &ClosedInterval<T>, w: &mut Witness<Fp>| {
            Ops::compare_closed_interval(value, x, w)
        };
        this.make_zcheck::<Ops, _, _>(default_fn, compare, w)
    }
}

pub trait ZkappCheckOps {
    fn compare_closed_interval<T: ForZkappCheck<Fp>>(
        interval: &ClosedInterval<T>,
        value: &T,
        w: &mut Witness<Fp>,
    ) -> Boolean;
    fn is_boolean_equal(a: &Boolean, b: &Boolean, w: &mut Witness<Fp>) -> Boolean;
    fn is_field_equal(a: &Fp, b: &Fp, w: &mut Witness<Fp>) -> Boolean;
    fn is_compressed_key_equal(
        a: &CompressedPubKey,
        b: &CompressedPubKey,
        w: &mut Witness<Fp>,
    ) -> Boolean;
    fn boolean_all<I>(bools: I, w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>;
    fn boolean_any<I>(bools: I, w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>;
}

pub struct InSnarkOps;
pub struct NonSnarkOps;

impl ZkappCheckOps for InSnarkOps {
    fn compare_closed_interval<T: ForZkappCheck<Fp>>(
        interval: &ClosedInterval<T>,
        value: &T,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let ClosedInterval { lower, upper } = interval;
        let lower = lower.to_checked();
        let upper = upper.to_checked();
        let x = value.to_checked();
        // We decompose this way because of OCaml evaluation order
        let lower_than_upper = <T as ForZkappCheck<Fp>>::lte(&x, &upper, w);
        let greater_than_lower = <T as ForZkappCheck<Fp>>::lte(&lower, &x, w);
        Boolean::all(&[greater_than_lower, lower_than_upper], w)
    }
    fn is_boolean_equal(a: &Boolean, b: &Boolean, w: &mut Witness<Fp>) -> Boolean {
        Boolean::equal(a, b, w)
    }
    fn is_field_equal(a: &Fp, b: &Fp, w: &mut Witness<Fp>) -> Boolean {
        field::equal(*a, *b, w)
    }
    fn is_compressed_key_equal(
        a: &CompressedPubKey,
        b: &CompressedPubKey,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        checked_equal_compressed_key(a, b, w)
    }
    fn boolean_all<I>(bools: I, w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>,
    {
        let bools = bools.into_iter().collect::<Vec<_>>();
        Boolean::all(&bools, w)
    }
    fn boolean_any<I>(bools: I, w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>,
    {
        let bools = bools.into_iter().collect::<Vec<_>>();
        Boolean::any(&bools, w)
    }
}

impl ZkappCheckOps for NonSnarkOps {
    fn compare_closed_interval<T: ForZkappCheck<Fp>>(
        interval: &ClosedInterval<T>,
        value: &T,
        _w: &mut Witness<Fp>,
    ) -> Boolean {
        let ClosedInterval { lower, upper } = interval;
        (lower <= value && value <= upper).to_boolean()
    }
    fn is_boolean_equal(a: &Boolean, b: &Boolean, _w: &mut Witness<Fp>) -> Boolean {
        (a == b).to_boolean()
    }
    fn is_field_equal(a: &Fp, b: &Fp, _w: &mut Witness<Fp>) -> Boolean {
        (a == b).to_boolean()
    }
    fn is_compressed_key_equal(
        a: &CompressedPubKey,
        b: &CompressedPubKey,
        _w: &mut Witness<Fp>,
    ) -> Boolean {
        (a == b).to_boolean()
    }
    fn boolean_all<I>(bools: I, _w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>,
    {
        bools.into_iter().all(|b| b.as_bool()).to_boolean()
    }
    fn boolean_any<I>(bools: I, _w: &mut Witness<Fp>) -> Boolean
    where
        I: IntoIterator<Item = Boolean>,
    {
        bools.into_iter().any(|b| b.as_bool()).to_boolean()
    }
}

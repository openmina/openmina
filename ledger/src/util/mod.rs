use std::ops::Neg;

use ark_ff::{BigInteger, PrimeField};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_signer::{CompressedPubKey, CurvePoint, Keypair, PubKey};

mod backtrace;
mod pubkey;

pub use pubkey::compressed_pubkey_from_address_maybe_with_error;

use crate::proofs::{field::FieldWitness, to_field_elements::ToFieldElements};
pub use crate::util::backtrace::*;

#[cfg(not(target_family = "wasm"))]
pub fn pid() -> u32 {
    std::process::id()
}

#[cfg(target_family = "wasm")]
pub fn pid() -> u32 {
    0
}

pub trait FpExt {
    fn to_decimal(&self) -> String;
}

impl FpExt for Fp {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
}

impl FpExt for Fq {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
}

pub fn gen_keypair() -> Keypair {
    let mut rng = rand::thread_rng();
    Keypair::rand(&mut rng)
}

pub fn gen_compressed() -> CompressedPubKey {
    gen_keypair().public.into_compressed()
}

/// Not sure if it's correct
/// I used the same code as there:
/// https://github.com/o1-labs/proof-systems/blob/226de4aeb11b8814327ab832e4fccdce5585f473/signer/src/pubkey.rs#L95-L106
pub fn decompress_pk(pk: &CompressedPubKey) -> Option<PubKey> {
    let y_parity = pk.is_odd;
    let x = pk.x;

    let mut pt = CurvePoint::get_point_from_x(x, y_parity)?;

    if pt.y.into_repr().is_even() == y_parity {
        pt.y = pt.y.neg();
    }

    if !pt.is_on_curve() {
        return None;
    }

    // Safe now because we checked point pt is on curve
    Some(PubKey::from_point_unsafe(pt))
}

pub fn take<T>(slice: &[T], n: usize) -> &[T] {
    slice.get(..n).unwrap_or(slice)
}

pub fn drop<T>(slice: &[T], n: usize) -> &[T] {
    slice.get(n..).unwrap_or(&[])
}

pub fn take_at<T>(slice: &[T], skip: usize, n: usize) -> &[T] {
    slice.get(skip..).map(|s| take(s, n)).unwrap_or(&[])
}

pub fn split_at<T>(slice: &[T], at: usize) -> (&[T], &[T]) {
    if at <= slice.len() {
        slice.split_at(at)
    } else {
        (slice, &[])
    }
}

pub fn split_at_vec<T>(mut vec: Vec<T>, at: usize) -> (Vec<T>, Vec<T>) {
    if at <= vec.len() {
        let vec2 = vec.split_off(at);
        (vec, vec2)
    } else {
        (vec, Vec::new())
    }
}

// `std::borrow::Cow` has a `ToOwned` constraints
pub enum MyCow<'a, T> {
    Borrow(&'a T),
    Own(T),
}

impl<'a, T> MyCow<'a, T> {
    pub fn borrow_or_default(v: &'a Option<T>) -> Self
    where
        T: Default,
    {
        match v.as_ref() {
            Some(v) => Self::Borrow(v),
            None => Self::Own(T::default()),
        }
    }

    pub fn borrow_or_else<F>(v: &'a Option<T>, default: F) -> Self
    where
        F: FnOnce() -> T,
    {
        match v.as_ref() {
            Some(v) => Self::Borrow(v),
            None => Self::Own(default()),
        }
    }
}

impl<'a, T> MyCow<'a, T>
where
    T: ToOwned<Owned = T>,
{
    pub fn to_owned(self) -> T {
        match self {
            MyCow::Borrow(v) => v.to_owned(),
            MyCow::Own(v) => v,
        }
    }
}

impl<'a, T> std::ops::Deref for MyCow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T> AsRef<T> for MyCow<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            MyCow::Borrow(v) => v,
            MyCow::Own(v) => v,
        }
    }
}

impl<'a, F, T> ToFieldElements<F> for MyCow<'a, T>
where
    F: FieldWitness,
    T: ToFieldElements<F>,
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let this: &T = self;
        this.to_field_elements(fields);
    }
}

// `std::borrow::Cow` has a `ToOwned` constraints
pub enum MyCowMut<'a, T> {
    Borrow(&'a mut T),
    Own(T),
}

impl<'a, T> std::ops::Deref for MyCowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MyCowMut::Borrow(v) => v,
            MyCowMut::Own(v) => v,
        }
    }
}

impl<'a, T> std::ops::DerefMut for MyCowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MyCowMut::Borrow(v) => v,
            MyCowMut::Own(v) => v,
        }
    }
}

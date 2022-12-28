use std::ops::Neg;

use ark_ff::{BigInteger, PrimeField};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_signer::{CompressedPubKey, CurvePoint, PubKey};

mod backtrace;
mod time;

pub use crate::util::backtrace::*;
pub use time::*;

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

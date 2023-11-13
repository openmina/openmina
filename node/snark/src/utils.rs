use ark_ff::{BigInteger256, Field, PrimeField};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use std::str::FromStr;

pub trait FpExt: Sized {
    fn to_decimal(&self) -> String;
    fn from_decimal(s: &str) -> Result<Self, String>;
}

impl FpExt for Fp {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
    fn from_decimal(s: &str) -> Result<Self, String> {
        num_bigint::BigUint::from_str(s)
            .map_err(|err| err.to_string())
            .and_then(|v| Self::from_repr(v.try_into()?).ok_or("from_repr failed".to_owned()))
    }
}

impl FpExt for Fq {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
    fn from_decimal(s: &str) -> Result<Self, String> {
        num_bigint::BigUint::from_str(s)
            .map_err(|err| err.to_string())
            .and_then(|v| Self::from_repr(v.try_into()?).ok_or("from_repr failed".to_owned()))
    }
}

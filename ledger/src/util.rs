use ark_ff::PrimeField;
use mina_hasher::Fp;

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

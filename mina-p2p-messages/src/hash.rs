use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_hasher::Fp;

pub trait MinaHash {
    fn try_hash(&self) -> Result<Fp, InvalidBigInt>;
}

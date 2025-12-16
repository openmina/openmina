use crate::bigint::InvalidBigInt;
use mina_curves::pasta::Fp;

pub trait MinaHash {
    fn try_hash(&self) -> Result<Fp, InvalidBigInt>;
}

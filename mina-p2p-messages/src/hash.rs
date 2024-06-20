use mina_hasher::Fp;

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

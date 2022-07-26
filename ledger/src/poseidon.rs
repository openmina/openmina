//! Mina Poseidon hasher
//!
//! An implementation of Mina's hasher based on the poseidon arithmetic sponge
//!
use std::marker::PhantomData;

// use crate::DomainParameter;
use mina_curves::pasta::Fp;
use mina_hasher::ROInput;
use oracle::{
    constants::{PlonkSpongeConstantsKimchi, PlonkSpongeConstantsLegacy, SpongeConstants},
    pasta,
    poseidon::{ArithmeticSponge, ArithmeticSpongeParams, Sponge, SpongeState},
};

use ark_ff::PrimeField;
use o1_utils::FieldHelpers;

/// The domain parameter trait is used during hashing to convey extra
/// arguments to domain string generation.  It is also used by generic signing code.
pub trait DomainParameter: Clone {
    /// Conversion into vector of bytes
    fn into_bytes(self) -> Vec<u8>;
}

impl DomainParameter for () {
    fn into_bytes(self) -> Vec<u8> {
        vec![]
    }
}

impl DomainParameter for u32 {
    fn into_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl DomainParameter for u64 {
    fn into_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

use super::{Hashable, Hasher};
// use super::{domain_prefix_to_field, Hashable, Hasher};

/// Transform domain prefix string to field element
fn domain_prefix_to_field<F: PrimeField>(prefix: String) -> F {
    const MAX_DOMAIN_STRING_LEN: usize = 20;
    assert!(prefix.len() <= MAX_DOMAIN_STRING_LEN);
    let prefix = &prefix[..std::cmp::min(prefix.len(), MAX_DOMAIN_STRING_LEN)];
    let bytes = prefix.to_string();
    // let bytes = format!("{:*<MAX_DOMAIN_STRING_LEN$}", prefix);

    println!("LAAAA {:?}", bytes);

    let mut bytes = bytes.as_bytes().to_vec();
    // let mut bytes = format!("{:*<MAX_DOMAIN_STRING_LEN$}", prefix)
    //     .as_bytes()
    //     .to_vec();
    bytes.resize(F::size_in_bytes(), 0);
    F::from_bytes(&bytes).expect("invalid domain bytes")
}

/// Poseidon hasher context
//
//  The arithmetic sponge parameters are large and costly to initialize,
//  so we only want to do this once and then re-use the Poseidon context
//  for many hashes. Also, following approach of the mina code we store
//  a backup of the initialized sponge state for efficient reuse.
pub struct Poseidon<SC: SpongeConstants, H: Hashable> {
    sponge: ArithmeticSponge<Fp, SC>,
    sponge_state: SpongeState,
    state: Vec<Fp>,
    phantom: PhantomData<H>,
}

impl<SC: SpongeConstants, H: Hashable> Poseidon<SC, H> {
    fn new(domain_param: H::D, sponge_params: &'static ArithmeticSpongeParams<Fp>) -> Self {
        let mut poseidon = Poseidon::<SC, H> {
            sponge: ArithmeticSponge::<Fp, SC>::new(sponge_params),
            sponge_state: SpongeState::Absorbed(0),
            state: vec![],
            phantom: PhantomData,
        };

        poseidon.init(domain_param);

        poseidon
    }
}

/// Poseidon hasher type with legacy plonk sponge constants
pub type PoseidonHasherLegacy<H> = Poseidon<PlonkSpongeConstantsLegacy, H>;

/// Create a legacy hasher context
pub(crate) fn new_legacy<H: Hashable>(domain_param: H::D) -> PoseidonHasherLegacy<H> {
    Poseidon::<PlonkSpongeConstantsLegacy, H>::new(domain_param, pasta::fp_legacy::static_params())
}

/// Poseidon hasher type with experimental kimchi plonk sponge constants
pub type PoseidonHasherKimchi<H> = Poseidon<PlonkSpongeConstantsKimchi, H>;

/// Create an experimental kimchi hasher context
pub(crate) fn new_kimchi<H: Hashable>(domain_param: H::D) -> PoseidonHasherKimchi<H> {
    Poseidon::<PlonkSpongeConstantsKimchi, H>::new(domain_param, pasta::fp_kimchi::static_params())
}

impl<SC: SpongeConstants, H: Hashable> Hasher<H> for Poseidon<SC, H>
// where
//     H::D: DomainParameter,
{
    fn reset(&mut self) -> &mut dyn Hasher<H> {
        // Efficient reset
        self.sponge.sponge_state = self.sponge_state.clone();
        self.sponge.state = self.state.clone();

        self
    }

    fn init(&mut self, domain_param: H::D) -> &mut dyn Hasher<H> {
        // Set sponge initial state and save it so the hasher context can be reused efficiently
        // N.B. Mina sets the sponge's initial state by hashing the input type's domain bytes
        self.sponge.reset();

        if let Some(domain_string) = H::domain_string(domain_param) {
            self.sponge
                .absorb(&[domain_prefix_to_field::<Fp>(domain_string)]);
            self.sponge.squeeze();
        }

        // Save initial state for efficient reset
        self.sponge_state = self.sponge.sponge_state.clone();
        self.state = self.sponge.state.clone();

        self
    }

    fn update(&mut self, input: &H) -> &mut dyn Hasher<H> {
        self.sponge.absorb(&input.to_roinput().to_fields());

        self
    }

    fn digest(&mut self) -> Fp {
        println!("STATE={:?}", self.sponge.state);
        println!("SPONGE_STATE={:?}", self.sponge.sponge_state.clone());
        let output = self.sponge.squeeze();
        self.sponge.reset();
        output
    }
}

#[derive(Clone, Debug, Default)]
struct ReceiptChainHash([u8; 32]);

fn empty_receipt_hash() {
    impl Hashable for ReceiptChainHash {
        type D = ();

        fn to_roinput(&self) -> ROInput {
            ROInput::new()
        }

        fn domain_string(domain_param: Self::D) -> Option<String> {
            Some("CodaReceiptEmpty".to_string())
        }
    }

    let mut hasher = new_kimchi::<ReceiptChainHash>(());
    // hasher.update(&ReceiptChainHash::default());
    let out = hasher.digest();

    println!("EMPTY_RECEIPT_HASH={:?}", out.to_string());

    let mut hasher = new_legacy::<ReceiptChainHash>(());
    hasher.update(&ReceiptChainHash::default());
    let out = hasher.digest();

    println!("EMPTY_RECEIPT_HASH={:?}", out.to_string());

    // let mut hasher = create_legacy::<ReceiptChainHash>(());
    // // hasher.update(&ReceiptChainHash::default());
    // let out = hasher.digest();
    // println!("EMPTY_RECEIPT_HASH={:?}", out.to_string());
    // out
}

#[cfg(test)]
mod tests {
    use mina_hasher::create_kimchi;

    use super::*;

    #[test]
    fn test_poseidon() {
        empty_receipt_hash();
    }
}

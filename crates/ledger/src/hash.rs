use mina_curves::pasta::{Fp, Fq};
use mina_signer::CompressedPubKey;

use crate::{proofs::witness::Witness, scan_state::currency};
use mina_hasher::{DomainParameter, Hashable, Hasher, ROInput};
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    pasta::FULL_ROUNDS,
    poseidon::{ArithmeticSponge, Sponge},
};

#[derive(Clone)]
pub struct CustomDomain(pub String);

impl DomainParameter for CustomDomain {
    fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Inputs(ROInput);

impl Inputs {
    pub fn new() -> Self {
        Self(ROInput::new())
    }

    pub fn append_field(&mut self, field: Fp) {
        self.0 = std::mem::take(&mut self.0).append_field(field);
    }

    pub fn append_u64(&mut self, value: u64) {
        self.0 = std::mem::take(&mut self.0).append_u64(value);
    }

    pub fn append_u32(&mut self, value: u32) {
        self.0 = std::mem::take(&mut self.0).append_u32(value);
    }

    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.0 = std::mem::take(&mut self.0).append_bytes(&value);
    }

    pub fn append_bool(&mut self, value: bool) {
        self.0 = std::mem::take(&mut self.0).append_bool(value);
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        self.0 = std::mem::take(&mut self.0).append_bytes(bytes);
    }

    pub fn to_fields(&self) -> Vec<Fp> {
        self.0.to_fields()
    }

    pub fn into_inner(self) -> ROInput {
        self.0
    }
}

pub use mina_core::HashParam;

pub fn hash_with_kimchi(domain: HashParam, fields: &[Fp]) -> Fp {
    let s: &'static str = domain.into();
    mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(s.to_string()))
        .update(&GenericHashable({
            let mut inputs = ROInput::new();
            for field in fields {
                inputs = inputs.append_field(*field);
            }
            inputs
        }))
        .digest()
}

pub(crate) trait KimchiParams: ark_ff::Field {
    fn static_params() -> &'static mina_poseidon::poseidon::ArithmeticSpongeParams<Self, FULL_ROUNDS>;
}

impl KimchiParams for Fp {
    fn static_params() -> &'static mina_poseidon::poseidon::ArithmeticSpongeParams<Self, FULL_ROUNDS>
    {
        mina_poseidon::pasta::fp_kimchi::static_params()
    }
}

impl KimchiParams for Fq {
    fn static_params() -> &'static mina_poseidon::poseidon::ArithmeticSpongeParams<Self, FULL_ROUNDS>
    {
        mina_poseidon::pasta::fq_kimchi::static_params()
    }
}

pub(crate) fn hash_fields<F: KimchiParams>(fields: &[F]) -> F {
    let mut sponge =
        ArithmeticSponge::<F, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(F::static_params());
    sponge.absorb(fields);
    sponge.squeeze()
}

/// Hash with no inputs (salt only). Uses zero-padded domain string to match
/// OCaml's `salt |> digest` behavior, which differs from the star-padded
/// domain used for hashing with inputs.
pub fn hash_noinputs(domain: HashParam) -> Fp {
    use ark_ff::Field;

    let s: &'static str = domain.into();
    let param_bytes = s.as_bytes();
    let mut bytes = [0u8; 32];
    bytes[..param_bytes.len()].copy_from_slice(param_bytes);
    let domain_field = Fp::from_random_bytes(&bytes).expect("invalid domain bytes");

    let mut sponge =
        ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(Fp::static_params());
    sponge.absorb(&[domain_field]);
    sponge.squeeze()
}

#[derive(Clone)]
pub struct GenericHashable(pub ROInput);

impl Hashable for GenericHashable {
    type D = CustomDomain;
    fn to_roinput(&self) -> ROInput {
        self.0.clone()
    }
    fn domain_string(domain: CustomDomain) -> Option<String> {
        Some(domain.0)
    }
}

pub trait ToInputs {
    fn to_inputs(&self, inputs: &mut Inputs);

    fn to_inputs_owned(&self) -> Inputs {
        let mut inputs = Inputs::new();
        self.to_inputs(&mut inputs);
        inputs
    }

    fn hash_with_param(&self, domain: HashParam) -> Fp {
        let inputs = self.to_inputs_owned();
        let s: &'static str = domain.into();
        mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(s.to_string()))
            .update(&GenericHashable(inputs.0))
            .digest()
    }

    fn checked_hash_with_param(&self, domain: HashParam, w: &mut Witness<Fp>) -> Fp {
        use crate::proofs::transaction::transaction_snark::checked_hash;

        let inputs = self.to_inputs_owned();
        checked_hash(domain, &inputs.to_fields(), w)
    }
}

impl ToInputs for Fp {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(*self);
    }
}

impl<const N: usize> ToInputs for [Fp; N] {
    fn to_inputs(&self, inputs: &mut Inputs) {
        for field in self {
            inputs.append_field(*field);
        }
    }
}

impl ToInputs for CompressedPubKey {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(self.x);
        inputs.append_bool(self.is_odd);
    }
}

impl ToInputs for crate::TokenId {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(self.0);
    }
}

impl ToInputs for bool {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_bool(*self);
    }
}

impl<T> ToInputs for currency::Signed<T>
where
    T: currency::Magnitude,
    T: ToInputs,
{
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/currency/currency.ml#L453>
    fn to_inputs(&self, inputs: &mut Inputs) {
        self.magnitude.to_inputs(inputs);
        let sgn = matches!(self.sgn, currency::Sgn::Pos);
        inputs.append_bool(sgn);
    }
}

pub trait AppendToInputs {
    fn append<T>(&mut self, value: &T)
    where
        T: ToInputs;
}

impl AppendToInputs for Inputs {
    fn append<T>(&mut self, value: &T)
    where
        T: ToInputs,
    {
        value.to_inputs(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use o1_utils::field_helpers::FieldHelpers;

    /// Verify Rust ArithmeticSponge matches snarky kimchi test vectors.
    /// Vectors from snarky/sponge/test_vectors/kimchi.json.
    #[test]
    fn test_sponge_kimchi_vectors() {
        // Test vector 0: empty input — just squeeze
        let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
            Fp::static_params(),
        );
        let result0 = sponge.squeeze();
        let expected0 =
            Fp::from_hex("a8eb9ee0f30046308abbfa5d20af73c81bbdabc25b459785024d045228bead2f")
                .unwrap();
        assert_eq!(result0, expected0, "empty input vector mismatch");

        // Test vector 1: 1 input
        let input1 =
            Fp::from_hex("f2eee8d8f6e5fb182c610cae6c5393fce69dc4d900e7b4923b074e54ad00fb36")
                .unwrap();
        let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
            Fp::static_params(),
        );
        sponge.absorb(&[input1]);
        let result1 = sponge.squeeze();
        let expected1 =
            Fp::from_hex("fb5992f65c07f9335995f43fd791d39012ad466717729e61045c297507054f3d")
                .unwrap();
        assert_eq!(result1, expected1, "1-input vector mismatch");

        // Test vector 2: 2 inputs
        let input2a =
            Fp::from_hex("bd3f1c8f183ceedea15080edbe79d30bd7d613b86bf2ba12007091c60ae39337")
                .unwrap();
        let input2b =
            Fp::from_hex("65e4f04ab87706bab06d13c7eee0a7807d0b8ce268b4ece6aab1e0508ec9c42f")
                .unwrap();
        let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
            Fp::static_params(),
        );
        sponge.absorb(&[input2a, input2b]);
        let result2 = sponge.squeeze();
        let expected2 =
            Fp::from_hex("fe2436f2027620a11233318b55d0a117086f09674826d1b7ce08d48ad0736c33")
                .unwrap();
        assert_eq!(result2, expected2, "2-input vector mismatch");
    }

    /// Verify hash_noinputs produces consistent results via manual sponge
    /// and via the hash_noinputs function.
    #[test]
    fn test_hash_noinputs_consistency() {
        use ark_ff::Field;

        let s = "CoinbaseStack";
        let param_bytes = s.as_bytes();
        let mut bytes = [0u8; 32];
        bytes[..param_bytes.len()].copy_from_slice(param_bytes);
        let domain_field = Fp::from_random_bytes(&bytes).expect("invalid domain bytes");

        // Verify byte-to-field: should be the raw LE byte representation
        assert_eq!(
            domain_field.to_hex(),
            "436f696e62617365537461636b00000000000000000000000000000000000000"
        );

        let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
            Fp::static_params(),
        );
        sponge.absorb(&[domain_field]);
        let result = sponge.squeeze();

        let result2 = hash_noinputs(HashParam::NoInputCoinbaseStack);
        assert_eq!(result, result2, "manual sponge != hash_noinputs");
    }
}

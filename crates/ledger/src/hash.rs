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
pub struct Inputs(pub ROInput);

impl Inputs {
    pub fn new() -> Self {
        Self(ROInput::new())
    }

    pub fn append_field(&mut self, field: Fp) {
        self.0 = self.0.clone().append_field(field);
    }

    pub fn append_u64(&mut self, value: u64) {
        self.0 = self.0.clone().append_u64(value);
    }

    pub fn append_u32(&mut self, value: u32) {
        self.0 = self.0.clone().append_u32(value);
    }

    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.0 = self.0.clone().append_bytes(&value);
    }

    pub fn append_bool(&mut self, value: bool) {
        self.0 = self.0.clone().append_bool(value);
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        self.0 = self.0.clone().append_bytes(bytes);
    }

    pub fn to_fields(&self) -> Vec<Fp> {
        self.0.to_fields()
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

pub fn hash_fields(fields: &[Fp]) -> Fp {
    let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
        mina_poseidon::pasta::fp_kimchi::static_params(),
    );
    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_fields_fq(fields: &[Fq]) -> Fq {
    let mut sponge = ArithmeticSponge::<Fq, PlonkSpongeConstantsKimchi, FULL_ROUNDS>::new(
        mina_poseidon::pasta::fq_kimchi::static_params(),
    );
    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_noinputs(domain: HashParam) -> Fp {
    let s: &'static str = domain.into();
    mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(s.to_string())).digest()
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

    #[test]
    fn test_inputs() {
        let mut inputs = Inputs::new();

        inputs.append_bool(true);
        inputs.append_u64(0); // initial_minimum_balance
        inputs.append_u32(0); // cliff_time
        inputs.append_u64(0); // cliff_amount
        inputs.append_u32(1); // vesting_period
        inputs.append_u64(0); // vesting_increment

        elog!("INPUTS={:?}", inputs);
        elog!("FIELDS={:?}", inputs.to_fields());
    }
}

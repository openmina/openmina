use mina_curves::pasta::{Fp, Fq};
use mina_signer::CompressedPubKey;

use crate::{proofs::witness::Witness, scan_state::currency};
use mina_hasher::{DomainParameter, Hashable, Hasher, ROInput};
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    poseidon::{ArithmeticSponge, Sponge},
};
use mina_poseidon::pasta::FULL_ROUNDS;

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

pub mod params {
    pub const MINA_ACCOUNT: &str = "MinaAccount";
    pub const MINA_PROTO_STATE: &str = "MinaProtoState";
    pub const MINA_PROTO_STATE_BODY: &str = "MinaProtoStateBody";
    pub const MINA_DERIVE_TOKEN_ID: &str = "MinaDeriveTokenId";
    pub const MINA_EPOCH_SEED: &str = "MinaEpochSeed";
    pub const MINA_SIDELOADED_VK: &str = "MinaSideLoadedVk";
    pub const MINA_VRF_MESSAGE: &str = "MinaVrfMessage";
    pub const MINA_VRF_OUTPUT: &str = "MinaVrfOutput";

    pub const CODA_RECEIPT_UC: &str = "CodaReceiptUC";
    pub const COINBASE_STACK: &str = "CoinbaseStack";

    pub const MINA_ACCOUNT_UPDATE_CONS: &str = "MinaAcctUpdateCons";
    pub const MINA_ACCOUNT_UPDATE_NODE: &str = "MinaAcctUpdateNode";
    pub const MINA_ACCOUNT_UPDATE_STACK_FRAME: &str = "MinaAcctUpdStckFrm";
    pub const MINA_ACCOUNT_UPDATE_STACK_FRAME_CONS: &str = "MinaActUpStckFrmCons";

    pub const MINA_ZKAPP_ACCOUNT: &str = "MinaZkappAccount";
    pub const MINA_ZKAPP_MEMO: &str = "MinaZkappMemo";
    pub const MINA_ZKAPP_URI: &str = "MinaZkappUri";
    pub const MINA_ZKAPP_EVENT: &str = "MinaZkappEvent";
    pub const MINA_ZKAPP_EVENTS: &str = "MinaZkappEvents";
    pub const MINA_ZKAPP_SEQ_EVENTS: &str = "MinaZkappSeqEvents";

    // devnet
    pub const CODA_SIGNATURE: &str = "CodaSignature";
    pub const TESTNET_ZKAPP_BODY: &str = "TestnetZkappBody";
    // mainnet
    pub const MINA_SIGNATURE_MAINNET: &str = "MinaSignatureMainnet";
    pub const MAINNET_ZKAPP_BODY: &str = "MainnetZkappBody";

    // Merkle Tree params
    pub const MINA_MERKLE_TREE_0: &str = "MinaMklTree000";
    pub const MINA_MERKLE_TREE_1: &str = "MinaMklTree001";
    pub const MINA_MERKLE_TREE_2: &str = "MinaMklTree002";
    pub const MINA_MERKLE_TREE_3: &str = "MinaMklTree003";
    pub const MINA_MERKLE_TREE_4: &str = "MinaMklTree004";
    pub const MINA_MERKLE_TREE_5: &str = "MinaMklTree005";
    pub const MINA_MERKLE_TREE_6: &str = "MinaMklTree006";
    pub const MINA_MERKLE_TREE_7: &str = "MinaMklTree007";
    pub const MINA_MERKLE_TREE_8: &str = "MinaMklTree008";
    pub const MINA_MERKLE_TREE_9: &str = "MinaMklTree009";
    pub const MINA_MERKLE_TREE_10: &str = "MinaMklTree010";
    pub const MINA_MERKLE_TREE_11: &str = "MinaMklTree011";
    pub const MINA_MERKLE_TREE_12: &str = "MinaMklTree012";
    pub const MINA_MERKLE_TREE_13: &str = "MinaMklTree013";
    pub const MINA_MERKLE_TREE_14: &str = "MinaMklTree014";
    pub const MINA_MERKLE_TREE_15: &str = "MinaMklTree015";
    pub const MINA_MERKLE_TREE_16: &str = "MinaMklTree016";
    pub const MINA_MERKLE_TREE_17: &str = "MinaMklTree017";
    pub const MINA_MERKLE_TREE_18: &str = "MinaMklTree018";
    pub const MINA_MERKLE_TREE_19: &str = "MinaMklTree019";
    pub const MINA_MERKLE_TREE_20: &str = "MinaMklTree020";
    pub const MINA_MERKLE_TREE_21: &str = "MinaMklTree021";
    pub const MINA_MERKLE_TREE_22: &str = "MinaMklTree022";
    pub const MINA_MERKLE_TREE_23: &str = "MinaMklTree023";
    pub const MINA_MERKLE_TREE_24: &str = "MinaMklTree024";
    pub const MINA_MERKLE_TREE_25: &str = "MinaMklTree025";
    pub const MINA_MERKLE_TREE_26: &str = "MinaMklTree026";
    pub const MINA_MERKLE_TREE_27: &str = "MinaMklTree027";
    pub const MINA_MERKLE_TREE_28: &str = "MinaMklTree028";
    pub const MINA_MERKLE_TREE_29: &str = "MinaMklTree029";
    pub const MINA_MERKLE_TREE_30: &str = "MinaMklTree030";
    pub const MINA_MERKLE_TREE_31: &str = "MinaMklTree031";
    pub const MINA_MERKLE_TREE_32: &str = "MinaMklTree032";
    pub const MINA_MERKLE_TREE_33: &str = "MinaMklTree033";
    pub const MINA_MERKLE_TREE_34: &str = "MinaMklTree034";
    pub const MINA_MERKLE_TREE_35: &str = "MinaMklTree035";

    pub const MINA_CB_MERKLE_TREE_0: &str = "MinaCbMklTree000";
    pub const MINA_CB_MERKLE_TREE_1: &str = "MinaCbMklTree001";
    pub const MINA_CB_MERKLE_TREE_2: &str = "MinaCbMklTree002";
    pub const MINA_CB_MERKLE_TREE_3: &str = "MinaCbMklTree003";
    pub const MINA_CB_MERKLE_TREE_4: &str = "MinaCbMklTree004";
    pub const MINA_CB_MERKLE_TREE_5: &str = "MinaCbMklTree005";

    pub const NO_INPUT_ZKAPP_ACTION_STATE_EMPTY_ELT: &str = "MinaZkappActionStateEmptyElt";
    pub const NO_INPUT_COINBASE_STACK: &str = "CoinbaseStack";
    pub const NO_INPUT_MINA_ZKAPP_EVENTS_EMPTY: &str = "MinaZkappEventsEmpty";
    pub const NO_INPUT_MINA_ZKAPP_ACTIONS_EMPTY: &str = "MinaZkappActionsEmpty";

    pub fn get_coinbase_param_for_height(height: usize) -> &'static str {
        match height {
            0 => MINA_CB_MERKLE_TREE_0,
            1 => MINA_CB_MERKLE_TREE_1,
            2 => MINA_CB_MERKLE_TREE_2,
            3 => MINA_CB_MERKLE_TREE_3,
            4 => MINA_CB_MERKLE_TREE_4,
            5 => MINA_CB_MERKLE_TREE_5,
            _ => panic!("Invalid height"),
        }
    }

    pub fn get_merkle_param_for_height(height: usize) -> &'static str {
        match height {
            0 => MINA_MERKLE_TREE_0,
            1 => MINA_MERKLE_TREE_1,
            2 => MINA_MERKLE_TREE_2,
            3 => MINA_MERKLE_TREE_3,
            4 => MINA_MERKLE_TREE_4,
            5 => MINA_MERKLE_TREE_5,
            6 => MINA_MERKLE_TREE_6,
            7 => MINA_MERKLE_TREE_7,
            8 => MINA_MERKLE_TREE_8,
            9 => MINA_MERKLE_TREE_9,
            10 => MINA_MERKLE_TREE_10,
            11 => MINA_MERKLE_TREE_11,
            12 => MINA_MERKLE_TREE_12,
            13 => MINA_MERKLE_TREE_13,
            14 => MINA_MERKLE_TREE_14,
            15 => MINA_MERKLE_TREE_15,
            16 => MINA_MERKLE_TREE_16,
            17 => MINA_MERKLE_TREE_17,
            18 => MINA_MERKLE_TREE_18,
            19 => MINA_MERKLE_TREE_19,
            20 => MINA_MERKLE_TREE_20,
            21 => MINA_MERKLE_TREE_21,
            22 => MINA_MERKLE_TREE_22,
            23 => MINA_MERKLE_TREE_23,
            24 => MINA_MERKLE_TREE_24,
            25 => MINA_MERKLE_TREE_25,
            26 => MINA_MERKLE_TREE_26,
            27 => MINA_MERKLE_TREE_27,
            28 => MINA_MERKLE_TREE_28,
            29 => MINA_MERKLE_TREE_29,
            30 => MINA_MERKLE_TREE_30,
            31 => MINA_MERKLE_TREE_31,
            32 => MINA_MERKLE_TREE_32,
            33 => MINA_MERKLE_TREE_33,
            34 => MINA_MERKLE_TREE_34,
            35 => MINA_MERKLE_TREE_35,
            _ => panic!("Invalid height"),
        }
    }
}

pub fn hash_with_kimchi(domain: &str, fields: &[Fp]) -> Fp {
    mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(domain.to_string()))
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

pub fn hash_noinputs(domain: &str) -> Fp {
    mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(domain.to_string()))
        .digest()
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

    fn hash_with_param(&self, domain: &str) -> Fp {
        let inputs = self.to_inputs_owned();
        mina_hasher::create_kimchi::<GenericHashable>(CustomDomain(domain.to_string()))
            .update(&GenericHashable(inputs.0))
            .digest()
    }

    fn checked_hash_with_param(&self, domain: &str, w: &mut Witness<Fp>) -> Fp {
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

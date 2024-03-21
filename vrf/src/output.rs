use ark_ff::{BigInteger, BigInteger256, PrimeField};
use mina_hasher::{create_kimchi, Hashable, Hasher, ROInput};
use mina_p2p_messages::v2::ConsensusVrfOutputTruncatedStableV1;
use num::{BigInt, BigRational, One, ToPrimitive};
use o1_utils::FieldHelpers;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{BaseField, ScalarField};

use super::serialize::{ark_deserialize, ark_serialize};

use super::message::VrfMessage;
use super::CurvePoint;

#[derive(Clone, Debug)]
pub struct VrfOutputHashInput {
    message: VrfMessage,
    g: CurvePoint,
}

impl VrfOutputHashInput {
    pub fn new(message: VrfMessage, g: CurvePoint) -> Self {
        Self { message, g }
    }
}

impl Hashable for VrfOutputHashInput {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new()
            .append_roinput(self.message.to_roinput())
            .append_field(self.g.x)
            .append_field(self.g.y)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        "MinaVrfOutput".to_string().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VrfOutput {
    message: VrfMessage,
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    output: CurvePoint,
}

impl VrfOutput {
    pub fn new(message: VrfMessage, output: CurvePoint) -> Self {
        Self { message, output }
    }

    pub fn raw(&self) -> CurvePoint {
        self.output
    }

    pub fn hash(&self) -> BaseField {
        let vrf_output_hash_input = VrfOutputHashInput::new(self.message.clone(), self.output);
        let mut hasher = create_kimchi::<VrfOutputHashInput>(());
        hasher.update(&vrf_output_hash_input).digest()
    }

    pub fn truncated(&self) -> ScalarField {
        let bits = self.hash().to_bits();

        let repr = BigInteger256::from_bits_le(&bits[..bits.len() - 3]);
        ScalarField::from_repr(repr).unwrap()
    }

    pub fn truncated_with_prefix_and_checksum(&self) -> Vec<u8> {
        let mut output_bytes = Vec::new();
        let prefix = vec![0x15, 0x20];

        output_bytes.extend(prefix);

        output_bytes.extend(self.truncated().to_bytes());

        // checksum
        let checksum_hash = Sha256::digest(&Sha256::digest(&output_bytes[..])[..]);
        output_bytes.extend(&checksum_hash[..4]);

        output_bytes
    }

    pub fn fractional(&self) -> f64 {
        // ocaml:   Bignum_bigint.(shift_left one length_in_bits))
        //          where: length_in_bits = Int.min 256 (Field.size_in_bits - 2)
        //                 Field.size_in_bits = 255
        let two_tpo_256 = BigInt::one() << 253u32;

        let vrf_out = BigInt::from_bytes_be(
            num::bigint::Sign::Plus,
            &self.truncated().into_repr().to_bytes_be(),
        );

        BigRational::new(vrf_out, two_tpo_256).to_f64().unwrap()
    }

    pub fn to_base_58(&self) -> String {
        let bytes = self.truncated_with_prefix_and_checksum();
        bs58::encode(bytes).into_string()
    }
}

impl std::fmt::Display for VrfOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = self.to_base_58();
        write!(f, "{encoded}")
    }
}

impl From<&VrfOutput> for ConsensusVrfOutputTruncatedStableV1 {
    fn from(value: &VrfOutput) -> Self {
        let bytes = value.truncated_with_prefix_and_checksum();
        Self(bytes.into())
    }
}

impl From<VrfOutput> for ConsensusVrfOutputTruncatedStableV1 {
    fn from(value: VrfOutput) -> Self {
        Self::from(&value)
    }
}

#[cfg(test)]
mod test {
    use mina_p2p_messages::v2::ConsensusVrfOutputTruncatedStableV1;

    use crate::{genesis_vrf, output::VrfOutput};

    #[test]
    fn test_serialization() {
        let vrf_output = genesis_vrf().unwrap();

        let serialized = serde_json::to_string(&vrf_output).unwrap();
        let deserialized: VrfOutput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vrf_output, deserialized);
    }

    #[test]
    fn test_conv_to_mina_type() {
        let vrf_output = genesis_vrf().unwrap();

        let converted = ConsensusVrfOutputTruncatedStableV1::from(vrf_output);
        let converted_string = serde_json::to_string_pretty(&converted).unwrap();
        let converted_string_deser: String = serde_json::from_str(&converted_string).unwrap();
        let expected = String::from("48H9Qk4D6RzS9kAJQX9HCDjiJ5qLiopxgxaS6xbDCWNaKQMQ9Y4C");

        assert_eq!(expected, converted_string_deser);
    }
}

use ark_ff::{BigInteger256, Field, One, Zero};

use mina_curves::pasta::curves::pallas::Pallas as CurvePoint;
use mina_hasher::{Hashable, Hasher, ROInput};
use mina_p2p_messages::v2::EpochSeed;
use num::BigUint;
use o1_utils::FieldHelpers;
use serde::{Deserialize, Serialize};

use super::{BaseField, VrfError, VrfResult};

pub const LEDGER_DEPTH: usize = 35;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VrfMessage {
    pub global_slot: u32,
    pub epoch_seed: EpochSeed,
    pub delegator_index: u64,
}

#[derive(Clone)]
struct VrfMessageHashable(ROInput);

impl Hashable for VrfMessageHashable {
    type D = ();
    fn to_roinput(&self) -> ROInput {
        self.0.clone()
    }
    fn domain_string(_: Self::D) -> Option<String> {
        Some("MinaVrfMessage".to_string())
    }
}

impl VrfMessage {
    pub fn new(global_slot: u32, epoch_seed: EpochSeed, delegator_index: u64) -> Self {
        Self {
            global_slot,
            epoch_seed,
            delegator_index,
        }
    }

    pub fn to_roinput(&self) -> ROInput {
        let mut inputs = ROInput::new();
        let epoch_seed = match self.epoch_seed.to_field() {
            Ok(epoch_seed) => epoch_seed,
            Err(_) => {
                // TODO: Return an error somehow
                mina_curves::pasta::Fp::zero()
            }
        };
        inputs = inputs.append_field(epoch_seed);

        // OCaml's Message.to_input includes seed as a field element and
        // global_slot + delegator as packed chunks.
        // Since mina_hasher::ROInput appends packed chunks AFTER all field elements,
        // we can just append them here and they will be in the correct relative order.
        // However, VrfOutput::hash needs to insert x and y BEFORE these packed chunks.

        // For VrfMessage::hash, we can just append them now.
        let packed_field = self.pack_slot_and_index();
        inputs = inputs.append_field(packed_field);
        inputs
    }

    pub(crate) fn pack_slot_and_index(&self) -> mina_curves::pasta::Fp {
        let mut packed = BigUint::from(self.global_slot);
        for i in 0..LEDGER_DEPTH {
            let bit = (self.delegator_index >> i) & 1 == 1;
            packed = (packed << 1) + (if bit { 1u64 } else { 0u64 });
        }

        mina_curves::pasta::Fp::new(BigInteger256::try_from(packed).unwrap())
    }

    pub fn hash(&self) -> BaseField {
        let mut hasher = mina_hasher::create_kimchi::<VrfMessageHashable>(());
        let inputs = self.to_roinput();
        hasher.update(&VrfMessageHashable(inputs));
        hasher.digest()
    }

    pub fn to_group(&self) -> VrfResult<CurvePoint> {
        // helpers
        let two = BaseField::one() + BaseField::one();
        let three = two + BaseField::one();

        // params, according to ocaml
        let mut projection_point_z_bytes =
            hex::decode("1AF731EC3CA2D77CC5D13EDC8C9A0A77978CB5F4FBFCC470B5983F5B6336DB69")?;
        projection_point_z_bytes.reverse();
        let projection_point_z = BaseField::from_bytes(&projection_point_z_bytes)?;
        let projection_point_y = BaseField::one();
        let conic_c = three;
        let u_over_2 = BaseField::one();
        let u = two;

        let t = self.hash();

        // field to conic
        let ct = conic_c * t;
        let s =
            two * ((ct * projection_point_y) + projection_point_z) / ((ct * t) + BaseField::one());
        let conic_z = projection_point_z - s;
        let conic_y = projection_point_y - (s * t);

        // conic to s
        let v = (conic_z / conic_y) - u_over_2;
        let y = conic_y;

        // s to v
        let x1 = v;
        let x2 = -(u + v);
        let x3 = u + (y * y);

        let get_y = |x: BaseField| -> Option<BaseField> {
            let five = BaseField::one()
                + BaseField::one()
                + BaseField::one()
                + BaseField::one()
                + BaseField::one();
            let mut res = x;
            res *= &x; // x^2
            res += BaseField::zero(); // x^2 + A x
            res *= &x; // x^3 + A x
            res += five; // x^3 + A x + B
            res.sqrt()
        };

        for x in [x1, x2, x3] {
            if let Some(y) = get_y(x) {
                return Ok(CurvePoint::new(x, y));
            }
        }

        Err(VrfError::ToGroupError(t))
    }
}

use ark_ff::{Field, One, Zero};

use ledger::{proofs::transaction::legacy_input::to_bits, ToInputs};
use mina_curves::pasta::curves::pallas::Pallas as CurvePoint;
use mina_p2p_messages::v2::EpochSeed;
use o1_utils::FieldHelpers;
use poseidon::hash::{params::MINA_VRF_MESSAGE, Inputs};
use serde::{Deserialize, Serialize};

use super::{BaseField, VrfError, VrfResult};

const LEDGER_DEPTH: usize = 35;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VrfMessage {
    global_slot: u32,
    epoch_seed: EpochSeed,
    delegator_index: u64,
}

impl VrfMessage {
    pub fn new(global_slot: u32, epoch_seed: EpochSeed, delegator_index: u64) -> Self {
        Self {
            global_slot,
            epoch_seed,
            delegator_index,
        }
    }

    pub fn hash(&self) -> BaseField {
        self.hash_with_param(&MINA_VRF_MESSAGE)
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

impl ToInputs for VrfMessage {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let epoch_seed = match self.epoch_seed.to_field() {
            Ok(epoch_seed) => epoch_seed,
            Err(_) => {
                // TODO: Return an error somehow
                mina_curves::pasta::Fp::zero()
            }
        };
        inputs.append_field(epoch_seed);
        inputs.append_u32(self.global_slot);
        for bit in to_bits::<_, LEDGER_DEPTH>(self.delegator_index) {
            inputs.append_bool(bit);
        }
    }
}

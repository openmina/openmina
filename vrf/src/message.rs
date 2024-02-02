// use o1_utils::FieldHelpers;
use mina_hasher::{Hashable, ROInput};
// use serde::{
//     de::{MapAccess, Visitor},
//     ser::SerializeMap,
//     Deserialize, Serialize,
// };

use super::BaseField;

const LEDGER_DEPTH: usize = 35;

#[derive(Clone, Debug, Default)]
pub struct VrfMessage {
    global_slot: u32,
    epoch_seed: BaseField,
    delegator_index: u64,
}

impl VrfMessage {
    pub fn new(global_slot: u32, epoch_seed: BaseField, delegator_index: u64) -> Self {
        Self {
            global_slot,
            epoch_seed,
            delegator_index,
        }
    }
}

impl Hashable for VrfMessage {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new().append_field(self.epoch_seed);

        for i in (0..LEDGER_DEPTH).rev() {
            roi = if self.delegator_index >> i & 1u64 == 1 {
                roi.append_bool(true)
            } else {
                roi.append_bool(false)
            };
        }

        roi = roi.append_u32(self.global_slot);
        roi
    }

    fn domain_string(_: Self::D) -> Option<String> {
        "MinaVrfMessage".to_string().into()
    }
}

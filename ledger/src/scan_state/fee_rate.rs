use crate::scan_state::currency::Magnitude;

use super::currency::Fee;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct FeeRate {
    q: fraction::Fraction,
}

impl FeeRate {
    pub fn make_exn(fee: Fee, weight: u64) -> Self {
        if weight == 0 {
            assert!(fee.is_zero());
        }

        Self {
            q: fraction::Fraction::new(fee.as_u64(), weight),
        }
    }
}

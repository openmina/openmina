use super::*;

impl From<SnarkBlockVerifyAction> for crate::Action {
    fn from(value: SnarkBlockVerifyAction) -> Self {
        Self::Snark(value.into())
    }
}

impl From<SnarkBlockVerifyEffectfulAction> for crate::Action {
    fn from(value: SnarkBlockVerifyEffectfulAction) -> Self {
        Self::Snark(value.into())
    }
}

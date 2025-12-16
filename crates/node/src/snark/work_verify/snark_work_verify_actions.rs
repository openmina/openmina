use super::*;

impl From<SnarkWorkVerifyAction> for crate::Action {
    fn from(value: SnarkWorkVerifyAction) -> Self {
        Self::Snark(value.into())
    }
}

impl From<SnarkWorkVerifyEffectfulAction> for crate::Action {
    fn from(value: SnarkWorkVerifyEffectfulAction) -> Self {
        Self::Snark(value.into())
    }
}

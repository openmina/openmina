use super::*;

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.snark, time)
    }
}

impl From<SnarkWorkVerifyAction> for crate::Action {
    fn from(value: SnarkWorkVerifyAction) -> Self {
        Self::Snark(value.into())
    }
}

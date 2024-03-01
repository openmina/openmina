use super::*;

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.snark, time)
    }
}

impl From<SnarkBlockVerifyAction> for crate::Action {
    fn from(value: SnarkBlockVerifyAction) -> Self {
        Self::Snark(value.into())
    }
}

use super::*;

impl redux::EnablingCondition<crate::State> for SnarkUserCommandVerifyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.snark, time)
    }
}

impl From<SnarkUserCommandVerifyAction> for crate::Action {
    fn from(value: SnarkUserCommandVerifyAction) -> Self {
        Self::Snark(value.into())
    }
}

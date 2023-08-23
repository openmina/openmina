use super::*;

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifyInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifyPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifyErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkWorkVerifyFinishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl From<SnarkWorkVerifyInitAction> for crate::Action {
    fn from(a: SnarkWorkVerifyInitAction) -> Self {
        Self::Snark(a.into())
    }
}

impl From<SnarkWorkVerifyErrorAction> for crate::Action {
    fn from(a: SnarkWorkVerifyErrorAction) -> Self {
        Self::Snark(a.into())
    }
}

impl From<SnarkWorkVerifySuccessAction> for crate::Action {
    fn from(a: SnarkWorkVerifySuccessAction) -> Self {
        Self::Snark(a.into())
    }
}

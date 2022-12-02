use super::*;
use block_verify::*;

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifyInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifyPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifyErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl redux::EnablingCondition<crate::State> for SnarkBlockVerifyFinishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.snark)
    }
}

impl From<SnarkBlockVerifyInitAction> for crate::Action {
    fn from(a: SnarkBlockVerifyInitAction) -> Self {
        Self::Snark(a.into())
    }
}

impl From<SnarkBlockVerifyErrorAction> for crate::Action {
    fn from(a: SnarkBlockVerifyErrorAction) -> Self {
        Self::Snark(a.into())
    }
}

impl From<SnarkBlockVerifySuccessAction> for crate::Action {
    fn from(a: SnarkBlockVerifySuccessAction) -> Self {
        Self::Snark(a.into())
    }
}

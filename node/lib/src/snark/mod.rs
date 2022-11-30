pub use ::snark::*;

mod snark_actions;
pub use snark_actions::*;

mod snark_effects;
pub use snark_effects::*;

impl<S> redux::SubStore<crate::State, SnarkState> for crate::Store<S>
where
    S: redux::Service,
{
    type SubAction = SnarkAction;
    type Service = S;

    fn state(&self) -> &SnarkState {
        &self.state.get().snark
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&SnarkState, &mut Self::Service) {
        (&self.state.get().snark, &mut self.service)
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<SnarkAction> + redux::EnablingCondition<crate::State>,
    {
        crate::Store::sub_dispatch(self, action)
    }
}

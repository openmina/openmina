pub use ::p2p::*;

pub mod connection;
pub mod pubsub;
pub mod rpc;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_effects;
pub use p2p_effects::*;

impl<S> redux::SubStore<crate::State, P2pState> for crate::Store<S>
where
    S: redux::Service,
{
    type SubAction = P2pAction;
    type Service = S;

    fn state(&self) -> &P2pState {
        &self.state.get().p2p
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&P2pState, &mut Self::Service) {
        (&self.state.get().p2p, &mut self.service)
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<P2pAction> + redux::EnablingCondition<crate::State>,
    {
        crate::Store::dispatch(self, action)
    }
}

impl<A: Into<P2pAction>> From<A> for crate::Action {
    fn from(value: A) -> Self {
        Self::P2p(value.into())
    }
}

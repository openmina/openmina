pub use ::node_p2p::*;

pub mod connection;
pub mod disconnection;
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
        crate::Store::sub_dispatch(self, action)
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::P2p(value.into())
            }
        }
    };
}

impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingRandomInitAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingInitAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingReconnectAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingPendingAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingErrorAction);
impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingSuccessAction);

impl_into_global_action!(disconnection::P2pDisconnectionInitAction);
impl_into_global_action!(disconnection::P2pDisconnectionFinishAction);

impl_into_global_action!(pubsub::P2pPubsubMessagePublishAction);
impl_into_global_action!(pubsub::P2pPubsubBytesPublishAction);
impl_into_global_action!(pubsub::P2pPubsubBytesReceivedAction);
impl_into_global_action!(pubsub::P2pPubsubMessageReceivedAction);

impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingInitAction);
impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingPendingAction);
impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingReceivedAction);
impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingErrorAction);
impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingSuccessAction);
impl_into_global_action!(rpc::outgoing::P2pRpcOutgoingFinishAction);

pub use ::p2p::*;

pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod network;
pub mod peer;

mod p2p_effects;
pub use p2p_effects::*;
use redux::EnablingCondition;

use crate::State;

impl<S> redux::SubStore<crate::State, P2pState> for crate::Store<S>
where
    S: redux::Service,
{
    type SubAction = P2pAction;
    type Service = S;

    fn state(&self) -> &P2pState {
        self.state
            .get()
            .p2p
            .ready()
            .expect("p2p should be initialized")
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&P2pState, &mut Self::Service) {
        (
            self
                .state
                .get()
                .p2p
                .ready()
                .expect("p2p should be initialized"),
            &mut self.service,
        )
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<P2pAction> + redux::EnablingCondition<P2pState>,
    {
        crate::Store::sub_dispatch(self, action)
    }
}

impl EnablingCondition<State> for P2pInitializeAction {
    fn is_enabled(&self, state: &State, _time: redux::Timestamp) -> bool {
        state.p2p.ready().is_none()
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

impl_into_global_action!(P2pInitializeAction);

impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingAction);

impl_into_global_action!(connection::incoming::P2pConnectionIncomingAction);

impl_into_global_action!(disconnection::P2pDisconnectionAction);

impl_into_global_action!(discovery::P2pDiscoveryAction);

impl_into_global_action!(network::P2pNetworkSchedulerAction);
impl_into_global_action!(network::kad::P2pNetworkKademliaAction);
impl_into_global_action!(network::pubsub::P2pNetworkPubsubAction);

impl_into_global_action!(channels::P2pChannelsMessageReceivedAction);

impl_into_global_action!(channels::best_tip::P2pChannelsBestTipAction);

impl_into_global_action!(channels::snark::P2pChannelsSnarkAction);

impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction);

impl_into_global_action!(channels::rpc::P2pChannelsRpcAction);

pub use ::p2p::*;
use p2p::{
    channels::{
        best_tip_effectful::P2pChannelsBestTipEffectfulAction,
        rpc_effectful::P2pChannelsRpcEffectfulAction,
        signaling::exchange_effectful::P2pChannelsSignalingExchangeEffectfulAction,
        snark_effectful::P2pChannelsSnarkEffectfulAction,
        snark_job_commitment_effectful::P2pChannelsSnarkJobCommitmentEffectfulAction,
        streaming_rpc_effectful::P2pChannelsStreamingRpcEffectfulAction,
        transaction_effectful::P2pChannelsTransactionEffectfulAction,
    },
    network::identify::stream_effectful::P2pNetworkIdentifyStreamEffectfulAction,
};

pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod network;
pub mod peer;

pub mod callbacks;

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
            self.state
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

    fn dispatch_callback<T>(&mut self, callback: redux::Callback<T>, args: T) -> bool
    where
        T: 'static,
        P2pAction: From<redux::AnyAction> + redux::EnablingCondition<P2pState>,
    {
        crate::Store::dispatch_callback(self, callback, args)
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
    (effectful $a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::P2pEffectful(value.into())
            }
        }
    };
}

impl_into_global_action!(P2pInitializeAction);

impl_into_global_action!(connection::outgoing::P2pConnectionOutgoingAction);

impl_into_global_action!(connection::incoming::P2pConnectionIncomingAction);

impl_into_global_action!(disconnection::P2pDisconnectionAction);

impl_into_global_action!(network::P2pNetworkSchedulerAction);
impl_into_global_action!(network::kad::P2pNetworkKademliaAction);
impl_into_global_action!(network::pubsub::P2pNetworkPubsubAction);

impl_into_global_action!(channels::P2pChannelsMessageReceivedAction);
impl_into_global_action!(channels::signaling::exchange::P2pChannelsSignalingExchangeAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipAction);
impl_into_global_action!(channels::transaction::P2pChannelsTransactionAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkAction);
impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcAction);
impl_into_global_action!(channels::streaming_rpc::P2pChannelsStreamingRpcAction);

impl_into_global_action!(p2p::P2pNetworkKademliaStreamAction);
impl_into_global_action!(p2p::P2pNetworkKadRequestAction);
impl_into_global_action!(p2p::P2pNetworkKadBootstrapAction);
impl_into_global_action!(p2p::P2pNetworkYamuxAction);
impl_into_global_action!(p2p::peer::P2pPeerAction);
impl_into_global_action!(p2p::network::identify::stream::P2pNetworkIdentifyStreamAction);
impl_into_global_action!(p2p::identify::P2pIdentifyAction);
impl_into_global_action!(p2p::P2pNetworkSelectAction);
impl_into_global_action!(p2p::P2pNetworkPnetAction);
impl_into_global_action!(p2p::P2pNetworkNoiseAction);
impl_into_global_action!(p2p::P2pNetworkRpcAction);

impl_into_global_action!(effectful network::kad_effectful::P2pNetworkKadEffectfulAction);
impl_into_global_action!(effectful p2p::P2pNetworkSchedulerEffectfulAction);
impl_into_global_action!(effectful p2p::P2pNetworkPnetEffectfulAction);
impl_into_global_action!(effectful connection::incoming_effectful::P2pConnectionIncomingEffectfulAction);
impl_into_global_action!(effectful connection::outgoing_effectful::P2pConnectionOutgoingEffectfulAction);
impl_into_global_action!(effectful p2p::disconnection_effectful::P2pDisconnectionEffectfulAction);
impl_into_global_action!(effectful P2pChannelsSignalingExchangeEffectfulAction);
impl_into_global_action!(effectful P2pChannelsBestTipEffectfulAction);
impl_into_global_action!(effectful P2pChannelsStreamingRpcEffectfulAction);
impl_into_global_action!(effectful P2pChannelsTransactionEffectfulAction);
impl_into_global_action!(effectful P2pChannelsSnarkJobCommitmentEffectfulAction);
impl_into_global_action!(effectful P2pChannelsRpcEffectfulAction);
impl_into_global_action!(effectful P2pChannelsSnarkEffectfulAction);
impl_into_global_action!(effectful network::pubsub::P2pNetworkPubsubEffectfulAction);
impl_into_global_action!(effectful P2pNetworkIdentifyStreamEffectfulAction);

impl p2p::P2pActionTrait<crate::State> for crate::Action {}

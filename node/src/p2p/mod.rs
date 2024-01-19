pub use ::p2p::*;

pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod listen;
pub mod peer;

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

impl_into_global_action!(listen::P2pListenNewAction);
impl_into_global_action!(listen::P2pListenExpiredAction);
impl_into_global_action!(listen::P2pListenErrorAction);
impl_into_global_action!(listen::P2pListenClosedAction);

impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingInitAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferReadyAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferSendSuccessAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingAnswerRecvPendingAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingAnswerRecvErrorAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingFinalizePendingAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingFinalizeErrorAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingFinalizeSuccessAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingTimeoutAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingErrorAction);
impl_into_global_action!(connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingSuccessAction);

impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingInitAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingAnswerReadyAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingAnswerSendSuccessAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingFinalizePendingAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingFinalizeErrorAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingFinalizeSuccessAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingTimeoutAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingErrorAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingSuccessAction);
impl_into_global_action!(connection::webrtc::incoming::P2pConnectionWebRTCIncomingLibp2pReceivedAction);

impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingInitAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingFinalizePendingAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingFinalizeSuccessAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingFinalizeErrorAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingFinalizeTimeoutAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingSuccessAction);
impl_into_global_action!(connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingErrorAction);

impl_into_global_action!(connection::libp2p::incoming::P2pConnectionLibP2pIncomingSuccessAction);

impl_into_global_action!(disconnection::P2pDisconnectionInitAction);
impl_into_global_action!(disconnection::P2pDisconnectionFinishAction);

impl_into_global_action!(discovery::P2pDiscoveryInitAction);
impl_into_global_action!(discovery::P2pDiscoverySuccessAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaBootstrapAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaInitAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaSuccessAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaFailureAction);
impl_into_global_action!(discovery::P2pDiscoveryKademliaAddRouteAction);

impl_into_global_action!(peer::P2pPeerAddLibP2pAction);
impl_into_global_action!(peer::P2pPeerAddWebRTCAction);
impl_into_global_action!(peer::P2pPeerReconnectAction);

impl_into_global_action!(channels::P2pChannelsMessageReceivedAction);

impl_into_global_action!(channels::best_tip::P2pChannelsBestTipInitAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipPendingAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipReadyAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipRequestSendAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipReceivedAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipRequestReceivedAction);
impl_into_global_action!(channels::best_tip::P2pChannelsBestTipResponseSendAction);

impl_into_global_action!(channels::snark::P2pChannelsSnarkInitAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkPendingAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkReadyAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkRequestSendAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkPromiseReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkRequestReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkResponseSendAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkLibp2pReceivedAction);
impl_into_global_action!(channels::snark::P2pChannelsSnarkLibp2pBroadcastAction);

impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentInitAction);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentPendingAction
);
impl_into_global_action!(channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentReadyAction);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestSendAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentPromiseReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestReceivedAction
);
impl_into_global_action!(
    channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentResponseSendAction
);

impl_into_global_action!(channels::rpc::P2pChannelsRpcInitAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcPendingAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcReadyAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcRequestSendAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcTimeoutAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcResponseReceivedAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcRequestReceivedAction);
impl_into_global_action!(channels::rpc::P2pChannelsRpcResponseSendAction);

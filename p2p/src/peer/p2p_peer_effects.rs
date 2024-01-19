use openmina_core::error;
use redux::{ActionMeta, EnablingCondition};

use crate::{
    channels::{
        best_tip::P2pChannelsBestTipInitAction, rpc::P2pChannelsRpcInitAction,
        snark::P2pChannelsSnarkInitAction,
        snark_job_commitment::P2pChannelsSnarkJobCommitmentInitAction, ChannelId,
    },
    connection::{
        libp2p::outgoing::P2pConnectionLibP2pOutgoingInitAction,
        webrtc::outgoing::P2pConnectionWebRTCOutgoingInitAction,
    },
    P2pPeerState,
};

use super::{
    P2pPeerAddLibP2pAction, P2pPeerAddWebRTCAction, P2pPeerReadyAction, P2pPeerReconnectAction,
};

impl P2pPeerAddLibP2pAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pConnectionLibP2pOutgoingInitAction: EnablingCondition<S>,
        P2pPeerAddLibP2pAction: EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionLibP2pOutgoingInitAction {
            peer_id: self.peer_id,
            rpc_id: self.rpc_id,
        });
    }
}

impl P2pPeerAddWebRTCAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pConnectionWebRTCOutgoingInitAction: EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingInitAction {
            peer_id: self.peer_id,
            rpc_id: self.rpc_id,
        });
    }
}

impl P2pPeerReconnectAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pConnectionLibP2pOutgoingInitAction: EnablingCondition<S>,
        P2pConnectionWebRTCOutgoingInitAction: EnablingCondition<S>,
    {
        match store.state().peers.get(&self.peer_id) {
            Some(P2pPeerState::Libp2p(_)) => {
                store.dispatch(P2pConnectionLibP2pOutgoingInitAction {
                    peer_id: self.peer_id,
                    rpc_id: None,
                });
            }
            Some(P2pPeerState::WebRTC(_)) => {
                store.dispatch(P2pConnectionWebRTCOutgoingInitAction {
                    peer_id: self.peer_id,
                    rpc_id: None,
                });
            }
            _ => {
                error!(meta.time(); "incorrect peer state {}", self.peer_id);
            }
        }
    }
}

impl P2pPeerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pChannelsBestTipInitAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkInitAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentInitAction: redux::EnablingCondition<S>,
        P2pChannelsRpcInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        // Dispatches can be done without a loop, but inside we do
        // exhaustive matching so that we don't miss any channels.
        for id in ChannelId::iter_all() {
            match id {
                ChannelId::BestTipPropagation => {
                    store.dispatch(P2pChannelsBestTipInitAction { peer_id });
                }
                ChannelId::SnarkPropagation => {
                    store.dispatch(P2pChannelsSnarkInitAction { peer_id });
                }
                ChannelId::SnarkJobCommitmentPropagation => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentInitAction { peer_id });
                }
                ChannelId::Rpc => {
                    store.dispatch(P2pChannelsRpcInitAction { peer_id });
                }
            }
        }
    }
}

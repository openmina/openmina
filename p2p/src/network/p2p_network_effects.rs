use crate::channels::rpc::P2pChannelsRpcAction;

use super::*;

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkPnetSetupNonceAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingTokenAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerSelectErrorAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerSelectDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseInitAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectOutgoingTokensAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseDecryptedDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseHandshakeDoneAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerYamuxDidInitAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxIncomingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxPingStreamAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOpenStreamAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkRpcInitAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkRpcOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingMessageAction: redux::EnablingCondition<S>,
        P2pNetworkRpcOutgoingQueryAction: redux::EnablingCondition<S>,
        P2pChannelsRpcAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::Scheduler(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
            Self::Select(v) => v.effects(meta, store),
            Self::Noise(v) => v.effects(meta, store),
            Self::Yamux(v) => v.effects(meta, store),
            Self::Rpc(v) => v.effects(meta, store),
        }
    }
}

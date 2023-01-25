use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use crate::consensus::{
    ConsensusAction, ConsensusBestTipHistoryUpdateAction, ConsensusBestTipUpdateAction,
    ConsensusBlockReceivedAction, ConsensusBlockSnarkVerifyPendingAction,
    ConsensusBlockSnarkVerifySuccessAction, ConsensusShortRangeForkResolveAction,
};
use crate::event_source::{
    EventSourceAction, EventSourceNewEventAction, EventSourceProcessEventsAction,
    EventSourceWaitForEventsAction, EventSourceWaitTimeoutAction,
};
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingErrorAction, P2pConnectionOutgoingInitAction,
    P2pConnectionOutgoingPendingAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingSuccessAction,
};
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::disconnection::{
    P2pDisconnectionAction, P2pDisconnectionFinishAction, P2pDisconnectionInitAction,
};
use crate::p2p::pubsub::{
    P2pPubsubAction, P2pPubsubBytesPublishAction, P2pPubsubBytesReceivedAction,
    P2pPubsubMessagePublishAction, P2pPubsubMessageReceivedAction,
};
use crate::p2p::rpc::outgoing::{
    P2pRpcOutgoingAction, P2pRpcOutgoingErrorAction, P2pRpcOutgoingFinishAction,
    P2pRpcOutgoingInitAction, P2pRpcOutgoingPendingAction, P2pRpcOutgoingReceivedAction,
    P2pRpcOutgoingSuccessAction,
};
use crate::p2p::rpc::P2pRpcAction;
use crate::p2p::{P2pAction, P2pPeerReadyAction};
use crate::rpc::{
    RpcAction, RpcActionStatsGetAction, RpcFinishAction, RpcGlobalStateGetAction,
    RpcP2pConnectionOutgoingErrorAction, RpcP2pConnectionOutgoingInitAction,
    RpcP2pConnectionOutgoingPendingAction, RpcP2pConnectionOutgoingSuccessAction,
    RpcP2pPubsubMessagePublishAction, RpcWatchedAccountsAddAction, RpcWatchedAccountsGetAction,
};
use crate::snark::block_verify::{
    SnarkBlockVerifyAction, SnarkBlockVerifyErrorAction, SnarkBlockVerifyFinishAction,
    SnarkBlockVerifyInitAction, SnarkBlockVerifyPendingAction, SnarkBlockVerifySuccessAction,
};
use crate::snark::SnarkAction;
use crate::watched_accounts::{
    WatchedAccountsAction, WatchedAccountsAddAction, WatchedAccountsBlockLedgerQueryInitAction,
    WatchedAccountsBlockLedgerQueryPendingAction, WatchedAccountsBlockLedgerQuerySuccessAction,
    WatchedAccountsBlockTransactionsIncludedAction,
    WatchedAccountsLedgerInitialStateGetErrorAction,
    WatchedAccountsLedgerInitialStateGetInitAction,
    WatchedAccountsLedgerInitialStateGetPendingAction,
    WatchedAccountsLedgerInitialStateGetRetryAction,
    WatchedAccountsLedgerInitialStateGetSuccessAction,
};
use crate::{Action, ActionKindGet, CheckTimeoutsAction};

#[derive(
    Serialize, Deserialize, TryFromPrimitive, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy,
)]
#[repr(u16)]
pub enum ActionKind {
    None,
    CheckTimeouts,
    ConsensusBestTipHistoryUpdate,
    ConsensusBestTipUpdate,
    ConsensusBlockReceived,
    ConsensusBlockSnarkVerifyPending,
    ConsensusBlockSnarkVerifySuccess,
    ConsensusShortRangeForkResolve,
    EventSourceNewEvent,
    EventSourceProcessEvents,
    EventSourceWaitForEvents,
    EventSourceWaitTimeout,
    P2pConnectionOutgoingError,
    P2pConnectionOutgoingInit,
    P2pConnectionOutgoingPending,
    P2pConnectionOutgoingReconnect,
    P2pConnectionOutgoingSuccess,
    P2pDisconnectionFinish,
    P2pDisconnectionInit,
    P2pPeerReady,
    P2pPubsubBytesPublish,
    P2pPubsubBytesReceived,
    P2pPubsubMessagePublish,
    P2pPubsubMessageReceived,
    P2pRpcOutgoingError,
    P2pRpcOutgoingFinish,
    P2pRpcOutgoingInit,
    P2pRpcOutgoingPending,
    P2pRpcOutgoingReceived,
    P2pRpcOutgoingSuccess,
    RpcActionStatsGet,
    RpcFinish,
    RpcGlobalStateGet,
    RpcP2pConnectionOutgoingError,
    RpcP2pConnectionOutgoingInit,
    RpcP2pConnectionOutgoingPending,
    RpcP2pConnectionOutgoingSuccess,
    RpcP2pPubsubMessagePublish,
    RpcWatchedAccountsAdd,
    RpcWatchedAccountsGet,
    SnarkBlockVerifyError,
    SnarkBlockVerifyFinish,
    SnarkBlockVerifyInit,
    SnarkBlockVerifyPending,
    SnarkBlockVerifySuccess,
    WatchedAccountsAdd,
    WatchedAccountsBlockLedgerQueryInit,
    WatchedAccountsBlockLedgerQueryPending,
    WatchedAccountsBlockLedgerQuerySuccess,
    WatchedAccountsBlockTransactionsIncluded,
    WatchedAccountsLedgerInitialStateGetError,
    WatchedAccountsLedgerInitialStateGetInit,
    WatchedAccountsLedgerInitialStateGetPending,
    WatchedAccountsLedgerInitialStateGetRetry,
    WatchedAccountsLedgerInitialStateGetSuccess,
}

impl ActionKindGet for Action {
    fn kind(&self) -> ActionKind {
        match self {
            Self::CheckTimeouts(a) => a.kind(),
            Self::EventSource(a) => a.kind(),
            Self::P2p(a) => a.kind(),
            Self::Snark(a) => a.kind(),
            Self::Consensus(a) => a.kind(),
            Self::Rpc(a) => a.kind(),
            Self::WatchedAccounts(a) => a.kind(),
        }
    }
}

impl ActionKindGet for CheckTimeoutsAction {
    fn kind(&self) -> ActionKind {
        ActionKind::CheckTimeouts
    }
}

impl ActionKindGet for EventSourceAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::ProcessEvents(a) => a.kind(),
            Self::NewEvent(a) => a.kind(),
            Self::WaitForEvents(a) => a.kind(),
            Self::WaitTimeout(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Connection(a) => a.kind(),
            Self::Disconnection(a) => a.kind(),
            Self::PeerReady(a) => a.kind(),
            Self::Pubsub(a) => a.kind(),
            Self::Rpc(a) => a.kind(),
        }
    }
}

impl ActionKindGet for SnarkAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::BlockVerify(a) => a.kind(),
        }
    }
}

impl ActionKindGet for ConsensusAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::BlockReceived(a) => a.kind(),
            Self::BlockSnarkVerifyPending(a) => a.kind(),
            Self::BlockSnarkVerifySuccess(a) => a.kind(),
            Self::ShortRangeForkResolve(a) => a.kind(),
            Self::BestTipUpdate(a) => a.kind(),
            Self::BestTipHistoryUpdate(a) => a.kind(),
        }
    }
}

impl ActionKindGet for RpcAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::GlobalStateGet(a) => a.kind(),
            Self::ActionStatsGet(a) => a.kind(),
            Self::P2pConnectionOutgoingInit(a) => a.kind(),
            Self::P2pConnectionOutgoingPending(a) => a.kind(),
            Self::P2pConnectionOutgoingError(a) => a.kind(),
            Self::P2pConnectionOutgoingSuccess(a) => a.kind(),
            Self::P2pPubsubMessagePublish(a) => a.kind(),
            Self::WatchedAccountsAdd(a) => a.kind(),
            Self::WatchedAccountsGet(a) => a.kind(),
            Self::Finish(a) => a.kind(),
        }
    }
}

impl ActionKindGet for WatchedAccountsAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Add(a) => a.kind(),
            Self::LedgerInitialStateGetInit(a) => a.kind(),
            Self::LedgerInitialStateGetPending(a) => a.kind(),
            Self::LedgerInitialStateGetError(a) => a.kind(),
            Self::LedgerInitialStateGetRetry(a) => a.kind(),
            Self::LedgerInitialStateGetSuccess(a) => a.kind(),
            Self::TransactionsIncludedInBlock(a) => a.kind(),
            Self::BlockLedgerQueryInit(a) => a.kind(),
            Self::BlockLedgerQueryPending(a) => a.kind(),
            Self::BlockLedgerQuerySuccess(a) => a.kind(),
        }
    }
}

impl ActionKindGet for EventSourceProcessEventsAction {
    fn kind(&self) -> ActionKind {
        ActionKind::EventSourceProcessEvents
    }
}

impl ActionKindGet for EventSourceNewEventAction {
    fn kind(&self) -> ActionKind {
        ActionKind::EventSourceNewEvent
    }
}

impl ActionKindGet for EventSourceWaitForEventsAction {
    fn kind(&self) -> ActionKind {
        ActionKind::EventSourceWaitForEvents
    }
}

impl ActionKindGet for EventSourceWaitTimeoutAction {
    fn kind(&self) -> ActionKind {
        ActionKind::EventSourceWaitTimeout
    }
}

impl ActionKindGet for P2pConnectionAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Outgoing(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pDisconnectionAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Finish(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pPeerReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPeerReady
    }
}

impl ActionKindGet for P2pPubsubAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::MessagePublish(a) => a.kind(),
            Self::BytesPublish(a) => a.kind(),
            Self::BytesReceived(a) => a.kind(),
            Self::MessageReceived(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pRpcAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Outgoing(a) => a.kind(),
        }
    }
}

impl ActionKindGet for SnarkBlockVerifyAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Error(a) => a.kind(),
            Self::Success(a) => a.kind(),
            Self::Finish(a) => a.kind(),
        }
    }
}

impl ActionKindGet for ConsensusBlockReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBlockReceived
    }
}

impl ActionKindGet for ConsensusBlockSnarkVerifyPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBlockSnarkVerifyPending
    }
}

impl ActionKindGet for ConsensusBlockSnarkVerifySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBlockSnarkVerifySuccess
    }
}

impl ActionKindGet for ConsensusShortRangeForkResolveAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusShortRangeForkResolve
    }
}

impl ActionKindGet for ConsensusBestTipUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBestTipUpdate
    }
}

impl ActionKindGet for ConsensusBestTipHistoryUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBestTipHistoryUpdate
    }
}

impl ActionKindGet for RpcGlobalStateGetAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcGlobalStateGet
    }
}

impl ActionKindGet for RpcActionStatsGetAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcActionStatsGet
    }
}

impl ActionKindGet for RpcP2pConnectionOutgoingInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionOutgoingInit
    }
}

impl ActionKindGet for RpcP2pConnectionOutgoingPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionOutgoingPending
    }
}

impl ActionKindGet for RpcP2pConnectionOutgoingErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionOutgoingError
    }
}

impl ActionKindGet for RpcP2pConnectionOutgoingSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionOutgoingSuccess
    }
}

impl ActionKindGet for RpcP2pPubsubMessagePublishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pPubsubMessagePublish
    }
}

impl ActionKindGet for RpcWatchedAccountsAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcWatchedAccountsAdd
    }
}

impl ActionKindGet for RpcWatchedAccountsGetAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcWatchedAccountsGet
    }
}

impl ActionKindGet for RpcFinishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcFinish
    }
}

impl ActionKindGet for WatchedAccountsAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsAdd
    }
}

impl ActionKindGet for WatchedAccountsLedgerInitialStateGetInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsLedgerInitialStateGetInit
    }
}

impl ActionKindGet for WatchedAccountsLedgerInitialStateGetPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsLedgerInitialStateGetPending
    }
}

impl ActionKindGet for WatchedAccountsLedgerInitialStateGetErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsLedgerInitialStateGetError
    }
}

impl ActionKindGet for WatchedAccountsLedgerInitialStateGetRetryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsLedgerInitialStateGetRetry
    }
}

impl ActionKindGet for WatchedAccountsLedgerInitialStateGetSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsLedgerInitialStateGetSuccess
    }
}

impl ActionKindGet for WatchedAccountsBlockTransactionsIncludedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsBlockTransactionsIncluded
    }
}

impl ActionKindGet for WatchedAccountsBlockLedgerQueryInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsBlockLedgerQueryInit
    }
}

impl ActionKindGet for WatchedAccountsBlockLedgerQueryPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsBlockLedgerQueryPending
    }
}

impl ActionKindGet for WatchedAccountsBlockLedgerQuerySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::WatchedAccountsBlockLedgerQuerySuccess
    }
}

impl ActionKindGet for P2pConnectionOutgoingAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Reconnect(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Error(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pDisconnectionInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pDisconnectionInit
    }
}

impl ActionKindGet for P2pDisconnectionFinishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pDisconnectionFinish
    }
}

impl ActionKindGet for P2pPubsubMessagePublishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPubsubMessagePublish
    }
}

impl ActionKindGet for P2pPubsubBytesPublishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPubsubBytesPublish
    }
}

impl ActionKindGet for P2pPubsubBytesReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPubsubBytesReceived
    }
}

impl ActionKindGet for P2pPubsubMessageReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPubsubMessageReceived
    }
}

impl ActionKindGet for P2pRpcOutgoingAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Received(a) => a.kind(),
            Self::Error(a) => a.kind(),
            Self::Success(a) => a.kind(),
            Self::Finish(a) => a.kind(),
        }
    }
}

impl ActionKindGet for SnarkBlockVerifyInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkBlockVerifyInit
    }
}

impl ActionKindGet for SnarkBlockVerifyPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkBlockVerifyPending
    }
}

impl ActionKindGet for SnarkBlockVerifyErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkBlockVerifyError
    }
}

impl ActionKindGet for SnarkBlockVerifySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkBlockVerifySuccess
    }
}

impl ActionKindGet for SnarkBlockVerifyFinishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkBlockVerifyFinish
    }
}

impl ActionKindGet for P2pConnectionOutgoingInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingInit
    }
}

impl ActionKindGet for P2pConnectionOutgoingReconnectAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingReconnect
    }
}

impl ActionKindGet for P2pConnectionOutgoingPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingPending
    }
}

impl ActionKindGet for P2pConnectionOutgoingErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingError
    }
}

impl ActionKindGet for P2pConnectionOutgoingSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingSuccess
    }
}

impl ActionKindGet for P2pRpcOutgoingInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingInit
    }
}

impl ActionKindGet for P2pRpcOutgoingPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingPending
    }
}

impl ActionKindGet for P2pRpcOutgoingReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingReceived
    }
}

impl ActionKindGet for P2pRpcOutgoingErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingError
    }
}

impl ActionKindGet for P2pRpcOutgoingSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingSuccess
    }
}

impl ActionKindGet for P2pRpcOutgoingFinishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pRpcOutgoingFinish
    }
}

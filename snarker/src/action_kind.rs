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
use crate::job_commitment::{
    JobCommitmentAction, JobCommitmentAddAction, JobCommitmentCheckTimeoutsAction,
    JobCommitmentCreateAction, JobCommitmentP2pSendAction, JobCommitmentP2pSendAllAction,
    JobCommitmentTimeoutAction,
};
use crate::ledger::{LedgerAction, LedgerChildAccountsAddAction, LedgerChildHashesAddAction};
use crate::p2p::channels::best_tip::{
    P2pChannelsBestTipAction, P2pChannelsBestTipInitAction, P2pChannelsBestTipPendingAction,
    P2pChannelsBestTipReadyAction, P2pChannelsBestTipReceivedAction,
    P2pChannelsBestTipRequestReceivedAction, P2pChannelsBestTipRequestSendAction,
    P2pChannelsBestTipResponseSendAction,
};
use crate::p2p::channels::rpc::{
    P2pChannelsRpcAction, P2pChannelsRpcInitAction, P2pChannelsRpcPendingAction,
    P2pChannelsRpcReadyAction, P2pChannelsRpcRequestReceivedAction,
    P2pChannelsRpcRequestSendAction, P2pChannelsRpcResponseReceivedAction,
    P2pChannelsRpcResponseSendAction, P2pChannelsRpcTimeoutAction,
};
use crate::p2p::channels::snark_job_commitment::{
    P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentInitAction,
    P2pChannelsSnarkJobCommitmentPendingAction, P2pChannelsSnarkJobCommitmentPromiseReceivedAction,
    P2pChannelsSnarkJobCommitmentReadyAction, P2pChannelsSnarkJobCommitmentReceivedAction,
    P2pChannelsSnarkJobCommitmentRequestReceivedAction,
    P2pChannelsSnarkJobCommitmentRequestSendAction,
    P2pChannelsSnarkJobCommitmentResponseSendAction,
};
use crate::p2p::channels::{P2pChannelsAction, P2pChannelsMessageReceivedAction};
use crate::p2p::connection::incoming::{
    P2pConnectionIncomingAction, P2pConnectionIncomingAnswerReadyAction,
    P2pConnectionIncomingAnswerSdpCreateErrorAction,
    P2pConnectionIncomingAnswerSdpCreatePendingAction,
    P2pConnectionIncomingAnswerSdpCreateSuccessAction,
    P2pConnectionIncomingAnswerSendSuccessAction, P2pConnectionIncomingErrorAction,
    P2pConnectionIncomingFinalizeErrorAction, P2pConnectionIncomingFinalizePendingAction,
    P2pConnectionIncomingFinalizeSuccessAction, P2pConnectionIncomingInitAction,
    P2pConnectionIncomingSuccessAction,
};
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingAnswerRecvErrorAction,
    P2pConnectionOutgoingAnswerRecvPendingAction, P2pConnectionOutgoingAnswerRecvSuccessAction,
    P2pConnectionOutgoingErrorAction, P2pConnectionOutgoingFinalizeErrorAction,
    P2pConnectionOutgoingFinalizePendingAction, P2pConnectionOutgoingFinalizeSuccessAction,
    P2pConnectionOutgoingInitAction, P2pConnectionOutgoingOfferReadyAction,
    P2pConnectionOutgoingOfferSdpCreateErrorAction,
    P2pConnectionOutgoingOfferSdpCreatePendingAction,
    P2pConnectionOutgoingOfferSdpCreateSuccessAction, P2pConnectionOutgoingOfferSendSuccessAction,
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingSuccessAction,
};
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::disconnection::{
    P2pDisconnectionAction, P2pDisconnectionFinishAction, P2pDisconnectionInitAction,
};
use crate::p2p::peer::{P2pPeerAction, P2pPeerBestTipUpdateAction, P2pPeerReadyAction};
use crate::p2p::P2pAction;
use crate::rpc::{
    RpcAction, RpcActionStatsGetAction, RpcFinishAction, RpcGlobalStateGetAction,
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingInitAction,
    RpcP2pConnectionIncomingPendingAction, RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingSuccessAction, RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingInitAction, RpcP2pConnectionOutgoingPendingAction,
    RpcP2pConnectionOutgoingSuccessAction, RpcSnarkerJobPickAndCommitAction,
};
use crate::snark::block_verify::{
    SnarkBlockVerifyAction, SnarkBlockVerifyErrorAction, SnarkBlockVerifyFinishAction,
    SnarkBlockVerifyInitAction, SnarkBlockVerifyPendingAction, SnarkBlockVerifySuccessAction,
};
use crate::snark::SnarkAction;
use crate::transition_frontier::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction,
    TransitionFrontierSyncLedgerSuccessAction,
};
use crate::transition_frontier::{
    TransitionFrontierAction, TransitionFrontierRootLedgerSyncPendingAction,
    TransitionFrontierSyncBestTipUpdateAction, TransitionFrontierSyncInitAction,
};
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
    JobCommitmentAdd,
    JobCommitmentCheckTimeouts,
    JobCommitmentCreate,
    JobCommitmentP2pSend,
    JobCommitmentP2pSendAll,
    JobCommitmentTimeout,
    LedgerChildAccountsAdd,
    LedgerChildHashesAdd,
    P2pChannelsBestTipInit,
    P2pChannelsBestTipPending,
    P2pChannelsBestTipReady,
    P2pChannelsBestTipReceived,
    P2pChannelsBestTipRequestReceived,
    P2pChannelsBestTipRequestSend,
    P2pChannelsBestTipResponseSend,
    P2pChannelsMessageReceived,
    P2pChannelsRpcInit,
    P2pChannelsRpcPending,
    P2pChannelsRpcReady,
    P2pChannelsRpcRequestReceived,
    P2pChannelsRpcRequestSend,
    P2pChannelsRpcResponseReceived,
    P2pChannelsRpcResponseSend,
    P2pChannelsRpcTimeout,
    P2pChannelsSnarkJobCommitmentInit,
    P2pChannelsSnarkJobCommitmentPending,
    P2pChannelsSnarkJobCommitmentPromiseReceived,
    P2pChannelsSnarkJobCommitmentReady,
    P2pChannelsSnarkJobCommitmentReceived,
    P2pChannelsSnarkJobCommitmentRequestReceived,
    P2pChannelsSnarkJobCommitmentRequestSend,
    P2pChannelsSnarkJobCommitmentResponseSend,
    P2pConnectionIncomingAnswerReady,
    P2pConnectionIncomingAnswerSdpCreateError,
    P2pConnectionIncomingAnswerSdpCreatePending,
    P2pConnectionIncomingAnswerSdpCreateSuccess,
    P2pConnectionIncomingAnswerSendSuccess,
    P2pConnectionIncomingError,
    P2pConnectionIncomingFinalizeError,
    P2pConnectionIncomingFinalizePending,
    P2pConnectionIncomingFinalizeSuccess,
    P2pConnectionIncomingInit,
    P2pConnectionIncomingSuccess,
    P2pConnectionOutgoingAnswerRecvError,
    P2pConnectionOutgoingAnswerRecvPending,
    P2pConnectionOutgoingAnswerRecvSuccess,
    P2pConnectionOutgoingError,
    P2pConnectionOutgoingFinalizeError,
    P2pConnectionOutgoingFinalizePending,
    P2pConnectionOutgoingFinalizeSuccess,
    P2pConnectionOutgoingInit,
    P2pConnectionOutgoingOfferReady,
    P2pConnectionOutgoingOfferSdpCreateError,
    P2pConnectionOutgoingOfferSdpCreatePending,
    P2pConnectionOutgoingOfferSdpCreateSuccess,
    P2pConnectionOutgoingOfferSendSuccess,
    P2pConnectionOutgoingRandomInit,
    P2pConnectionOutgoingReconnect,
    P2pConnectionOutgoingSuccess,
    P2pDisconnectionFinish,
    P2pDisconnectionInit,
    P2pPeerBestTipUpdate,
    P2pPeerReady,
    RpcActionStatsGet,
    RpcFinish,
    RpcGlobalStateGet,
    RpcP2pConnectionIncomingError,
    RpcP2pConnectionIncomingInit,
    RpcP2pConnectionIncomingPending,
    RpcP2pConnectionIncomingRespond,
    RpcP2pConnectionIncomingSuccess,
    RpcP2pConnectionOutgoingError,
    RpcP2pConnectionOutgoingInit,
    RpcP2pConnectionOutgoingPending,
    RpcP2pConnectionOutgoingSuccess,
    RpcSnarkerJobPickAndCommit,
    SnarkBlockVerifyError,
    SnarkBlockVerifyFinish,
    SnarkBlockVerifyInit,
    SnarkBlockVerifyPending,
    SnarkBlockVerifySuccess,
    TransitionFrontierRootLedgerSyncPending,
    TransitionFrontierSyncBestTipUpdate,
    TransitionFrontierSyncInit,
    TransitionFrontierSyncLedgerInit,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceived,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceived,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryError,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInit,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPending,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetry,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccess,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQuery,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPending,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccess,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchError,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchInit,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchPending,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccess,
    TransitionFrontierSyncLedgerStagedLedgerReconstructPending,
    TransitionFrontierSyncLedgerStagedLedgerReconstructSuccess,
    TransitionFrontierSyncLedgerSuccess,
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

impl ActionKind {
    pub const COUNT: usize = 124;
}

impl ActionKindGet for Action {
    fn kind(&self) -> ActionKind {
        match self {
            Self::CheckTimeouts(a) => a.kind(),
            Self::EventSource(a) => a.kind(),
            Self::Ledger(a) => a.kind(),
            Self::P2p(a) => a.kind(),
            Self::Snark(a) => a.kind(),
            Self::Consensus(a) => a.kind(),
            Self::TransitionFrontier(a) => a.kind(),
            Self::JobCommitment(a) => a.kind(),
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

impl ActionKindGet for LedgerAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::ChildHashesAdd(a) => a.kind(),
            Self::ChildAccountsAdd(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Connection(a) => a.kind(),
            Self::Disconnection(a) => a.kind(),
            Self::Channels(a) => a.kind(),
            Self::Peer(a) => a.kind(),
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

impl ActionKindGet for TransitionFrontierAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::SyncInit(a) => a.kind(),
            Self::SyncBestTipUpdate(a) => a.kind(),
            Self::RootLedgerSyncPending(a) => a.kind(),
            Self::SyncLedger(a) => a.kind(),
        }
    }
}

impl ActionKindGet for JobCommitmentAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Create(a) => a.kind(),
            Self::Add(a) => a.kind(),
            Self::P2pSendAll(a) => a.kind(),
            Self::P2pSend(a) => a.kind(),
            Self::CheckTimeouts(a) => a.kind(),
            Self::Timeout(a) => a.kind(),
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
            Self::P2pConnectionIncomingInit(a) => a.kind(),
            Self::P2pConnectionIncomingPending(a) => a.kind(),
            Self::P2pConnectionIncomingRespond(a) => a.kind(),
            Self::P2pConnectionIncomingError(a) => a.kind(),
            Self::P2pConnectionIncomingSuccess(a) => a.kind(),
            Self::SnarkerJobPickAndCommit(a) => a.kind(),
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

impl ActionKindGet for LedgerChildHashesAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::LedgerChildHashesAdd
    }
}

impl ActionKindGet for LedgerChildAccountsAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::LedgerChildAccountsAdd
    }
}

impl ActionKindGet for P2pConnectionAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Outgoing(a) => a.kind(),
            Self::Incoming(a) => a.kind(),
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

impl ActionKindGet for P2pChannelsAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::MessageReceived(a) => a.kind(),
            Self::BestTip(a) => a.kind(),
            Self::SnarkJobCommitment(a) => a.kind(),
            Self::Rpc(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pPeerAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Ready(a) => a.kind(),
            Self::BestTipUpdate(a) => a.kind(),
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

impl ActionKindGet for TransitionFrontierSyncInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncInit
    }
}

impl ActionKindGet for TransitionFrontierSyncBestTipUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBestTipUpdate
    }
}

impl ActionKindGet for TransitionFrontierRootLedgerSyncPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierRootLedgerSyncPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::SnarkedLedgerSyncPending(a) => a.kind(),
            Self::SnarkedLedgerSyncPeersQuery(a) => a.kind(),
            Self::SnarkedLedgerSyncPeerQueryInit(a) => a.kind(),
            Self::SnarkedLedgerSyncPeerQueryPending(a) => a.kind(),
            Self::SnarkedLedgerSyncPeerQueryRetry(a) => a.kind(),
            Self::SnarkedLedgerSyncPeerQueryError(a) => a.kind(),
            Self::SnarkedLedgerSyncPeerQuerySuccess(a) => a.kind(),
            Self::SnarkedLedgerSyncChildHashesReceived(a) => a.kind(),
            Self::SnarkedLedgerSyncChildAccountsReceived(a) => a.kind(),
            Self::SnarkedLedgerSyncSuccess(a) => a.kind(),
            Self::StagedLedgerReconstructPending(a) => a.kind(),
            Self::StagedLedgerPartsFetchInit(a) => a.kind(),
            Self::StagedLedgerPartsFetchPending(a) => a.kind(),
            Self::StagedLedgerPartsFetchError(a) => a.kind(),
            Self::StagedLedgerPartsFetchSuccess(a) => a.kind(),
            Self::StagedLedgerReconstructSuccess(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
    }
}

impl ActionKindGet for JobCommitmentCreateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentCreate
    }
}

impl ActionKindGet for JobCommitmentAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentAdd
    }
}

impl ActionKindGet for JobCommitmentP2pSendAllAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentP2pSendAll
    }
}

impl ActionKindGet for JobCommitmentP2pSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentP2pSend
    }
}

impl ActionKindGet for JobCommitmentCheckTimeoutsAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentCheckTimeouts
    }
}

impl ActionKindGet for JobCommitmentTimeoutAction {
    fn kind(&self) -> ActionKind {
        ActionKind::JobCommitmentTimeout
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

impl ActionKindGet for RpcP2pConnectionIncomingInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionIncomingInit
    }
}

impl ActionKindGet for RpcP2pConnectionIncomingPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionIncomingPending
    }
}

impl ActionKindGet for RpcP2pConnectionIncomingRespondAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionIncomingRespond
    }
}

impl ActionKindGet for RpcP2pConnectionIncomingErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionIncomingError
    }
}

impl ActionKindGet for RpcP2pConnectionIncomingSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcP2pConnectionIncomingSuccess
    }
}

impl ActionKindGet for RpcSnarkerJobPickAndCommitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcSnarkerJobPickAndCommit
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
            Self::RandomInit(a) => a.kind(),
            Self::Init(a) => a.kind(),
            Self::Reconnect(a) => a.kind(),
            Self::OfferSdpCreatePending(a) => a.kind(),
            Self::OfferSdpCreateError(a) => a.kind(),
            Self::OfferSdpCreateSuccess(a) => a.kind(),
            Self::OfferReady(a) => a.kind(),
            Self::OfferSendSuccess(a) => a.kind(),
            Self::AnswerRecvPending(a) => a.kind(),
            Self::AnswerRecvError(a) => a.kind(),
            Self::AnswerRecvSuccess(a) => a.kind(),
            Self::FinalizePending(a) => a.kind(),
            Self::FinalizeError(a) => a.kind(),
            Self::FinalizeSuccess(a) => a.kind(),
            Self::Error(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pConnectionIncomingAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::AnswerSdpCreatePending(a) => a.kind(),
            Self::AnswerSdpCreateError(a) => a.kind(),
            Self::AnswerSdpCreateSuccess(a) => a.kind(),
            Self::AnswerReady(a) => a.kind(),
            Self::AnswerSendSuccess(a) => a.kind(),
            Self::FinalizePending(a) => a.kind(),
            Self::FinalizeError(a) => a.kind(),
            Self::FinalizeSuccess(a) => a.kind(),
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

impl ActionKindGet for P2pChannelsMessageReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsMessageReceived
    }
}

impl ActionKindGet for P2pChannelsBestTipAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Ready(a) => a.kind(),
            Self::RequestSend(a) => a.kind(),
            Self::Received(a) => a.kind(),
            Self::RequestReceived(a) => a.kind(),
            Self::ResponseSend(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Ready(a) => a.kind(),
            Self::RequestSend(a) => a.kind(),
            Self::PromiseReceived(a) => a.kind(),
            Self::Received(a) => a.kind(),
            Self::RequestReceived(a) => a.kind(),
            Self::ResponseSend(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pChannelsRpcAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Pending(a) => a.kind(),
            Self::Ready(a) => a.kind(),
            Self::RequestSend(a) => a.kind(),
            Self::Timeout(a) => a.kind(),
            Self::ResponseReceived(a) => a.kind(),
            Self::RequestReceived(a) => a.kind(),
            Self::ResponseSend(a) => a.kind(),
        }
    }
}

impl ActionKindGet for P2pPeerReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPeerReady
    }
}

impl ActionKindGet for P2pPeerBestTipUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPeerBestTipUpdate
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

impl ActionKindGet for TransitionFrontierSyncLedgerInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQuery
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetry
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryError
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceived
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceived
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerReconstructPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerPartsFetchInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerPartsFetchPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerPartsFetchError
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedLedgerReconstructSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSuccess
    }
}

impl ActionKindGet for P2pConnectionOutgoingRandomInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingRandomInit
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

impl ActionKindGet for P2pConnectionOutgoingOfferSdpCreatePendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingOfferSdpCreatePending
    }
}

impl ActionKindGet for P2pConnectionOutgoingOfferSdpCreateErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingOfferSdpCreateError
    }
}

impl ActionKindGet for P2pConnectionOutgoingOfferSdpCreateSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingOfferSdpCreateSuccess
    }
}

impl ActionKindGet for P2pConnectionOutgoingOfferReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingOfferReady
    }
}

impl ActionKindGet for P2pConnectionOutgoingOfferSendSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingOfferSendSuccess
    }
}

impl ActionKindGet for P2pConnectionOutgoingAnswerRecvPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingAnswerRecvPending
    }
}

impl ActionKindGet for P2pConnectionOutgoingAnswerRecvErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingAnswerRecvError
    }
}

impl ActionKindGet for P2pConnectionOutgoingAnswerRecvSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingAnswerRecvSuccess
    }
}

impl ActionKindGet for P2pConnectionOutgoingFinalizePendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingFinalizePending
    }
}

impl ActionKindGet for P2pConnectionOutgoingFinalizeErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingFinalizeError
    }
}

impl ActionKindGet for P2pConnectionOutgoingFinalizeSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionOutgoingFinalizeSuccess
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

impl ActionKindGet for P2pConnectionIncomingInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingInit
    }
}

impl ActionKindGet for P2pConnectionIncomingAnswerSdpCreatePendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingAnswerSdpCreatePending
    }
}

impl ActionKindGet for P2pConnectionIncomingAnswerSdpCreateErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingAnswerSdpCreateError
    }
}

impl ActionKindGet for P2pConnectionIncomingAnswerSdpCreateSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingAnswerSdpCreateSuccess
    }
}

impl ActionKindGet for P2pConnectionIncomingAnswerReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingAnswerReady
    }
}

impl ActionKindGet for P2pConnectionIncomingAnswerSendSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingAnswerSendSuccess
    }
}

impl ActionKindGet for P2pConnectionIncomingFinalizePendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingFinalizePending
    }
}

impl ActionKindGet for P2pConnectionIncomingFinalizeErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingFinalizeError
    }
}

impl ActionKindGet for P2pConnectionIncomingFinalizeSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingFinalizeSuccess
    }
}

impl ActionKindGet for P2pConnectionIncomingErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingError
    }
}

impl ActionKindGet for P2pConnectionIncomingSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pConnectionIncomingSuccess
    }
}

impl ActionKindGet for P2pChannelsBestTipInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipInit
    }
}

impl ActionKindGet for P2pChannelsBestTipPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipPending
    }
}

impl ActionKindGet for P2pChannelsBestTipReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipReady
    }
}

impl ActionKindGet for P2pChannelsBestTipRequestSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipRequestSend
    }
}

impl ActionKindGet for P2pChannelsBestTipReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipReceived
    }
}

impl ActionKindGet for P2pChannelsBestTipRequestReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipRequestReceived
    }
}

impl ActionKindGet for P2pChannelsBestTipResponseSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsBestTipResponseSend
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentInit
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentPending
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentReady
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentRequestSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentRequestSend
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentPromiseReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentPromiseReceived
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentReceived
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentRequestReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentRequestReceived
    }
}

impl ActionKindGet for P2pChannelsSnarkJobCommitmentResponseSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsSnarkJobCommitmentResponseSend
    }
}

impl ActionKindGet for P2pChannelsRpcInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcInit
    }
}

impl ActionKindGet for P2pChannelsRpcPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcPending
    }
}

impl ActionKindGet for P2pChannelsRpcReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcReady
    }
}

impl ActionKindGet for P2pChannelsRpcRequestSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcRequestSend
    }
}

impl ActionKindGet for P2pChannelsRpcTimeoutAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcTimeout
    }
}

impl ActionKindGet for P2pChannelsRpcResponseReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcResponseReceived
    }
}

impl ActionKindGet for P2pChannelsRpcRequestReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcRequestReceived
    }
}

impl ActionKindGet for P2pChannelsRpcResponseSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pChannelsRpcResponseSend
    }
}

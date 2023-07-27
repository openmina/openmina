use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use crate::consensus::{
    ConsensusAction, ConsensusBestTipUpdateAction, ConsensusBlockChainProofUpdateAction,
    ConsensusBlockReceivedAction, ConsensusBlockSnarkVerifyPendingAction,
    ConsensusBlockSnarkVerifySuccessAction, ConsensusDetectForkRangeAction,
    ConsensusLongRangeForkResolveAction, ConsensusShortRangeForkResolveAction,
};
use crate::event_source::{
    EventSourceAction, EventSourceNewEventAction, EventSourceProcessEventsAction,
    EventSourceWaitForEventsAction, EventSourceWaitTimeoutAction,
};
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
    RpcP2pConnectionOutgoingSuccessAction, RpcSnarkPoolAvailableJobsGetAction,
    RpcSnarkerJobCommitAction, RpcSyncStatsGetAction,
};
use crate::snark::block_verify::{
    SnarkBlockVerifyAction, SnarkBlockVerifyErrorAction, SnarkBlockVerifyFinishAction,
    SnarkBlockVerifyInitAction, SnarkBlockVerifyPendingAction, SnarkBlockVerifySuccessAction,
};
use crate::snark::SnarkAction;
use crate::snark_pool::{
    SnarkPoolAction, SnarkPoolCheckTimeoutsAction, SnarkPoolCommitmentCreateAction,
    SnarkPoolJobCommitmentAddAction, SnarkPoolJobCommitmentTimeoutAction,
    SnarkPoolJobsUpdateAction, SnarkPoolP2pSendAction, SnarkPoolP2pSendAllAction,
};
use crate::transition_frontier::sync::ledger::snarked::{
    TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction,
    TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction,
    TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction,
    TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction,
    TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction,
    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedPeersQueryAction,
    TransitionFrontierSyncLedgerSnarkedPendingAction,
    TransitionFrontierSyncLedgerSnarkedSuccessAction,
};
use crate::transition_frontier::sync::ledger::staged::{
    TransitionFrontierSyncLedgerStagedAction,
    TransitionFrontierSyncLedgerStagedPartsFetchPendingAction,
    TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction,
    TransitionFrontierSyncLedgerStagedPartsPeerValidAction,
    TransitionFrontierSyncLedgerStagedReconstructErrorAction,
    TransitionFrontierSyncLedgerStagedReconstructInitAction,
    TransitionFrontierSyncLedgerStagedReconstructPendingAction,
    TransitionFrontierSyncLedgerStagedReconstructSuccessAction,
    TransitionFrontierSyncLedgerStagedSuccessAction,
};
use crate::transition_frontier::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSuccessAction,
};
use crate::transition_frontier::sync::{
    TransitionFrontierSyncAction, TransitionFrontierSyncBestTipUpdateAction,
    TransitionFrontierSyncBlocksFetchSuccessAction,
    TransitionFrontierSyncBlocksNextApplyInitAction,
    TransitionFrontierSyncBlocksNextApplyPendingAction,
    TransitionFrontierSyncBlocksNextApplySuccessAction,
    TransitionFrontierSyncBlocksPeerQueryErrorAction,
    TransitionFrontierSyncBlocksPeerQueryInitAction,
    TransitionFrontierSyncBlocksPeerQueryPendingAction,
    TransitionFrontierSyncBlocksPeerQueryRetryAction,
    TransitionFrontierSyncBlocksPeerQuerySuccessAction,
    TransitionFrontierSyncBlocksPeersQueryAction, TransitionFrontierSyncBlocksPendingAction,
    TransitionFrontierSyncBlocksSuccessAction, TransitionFrontierSyncInitAction,
    TransitionFrontierSyncLedgerRootPendingAction, TransitionFrontierSyncLedgerRootSuccessAction,
};
use crate::transition_frontier::{TransitionFrontierAction, TransitionFrontierSyncedAction};
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
    ConsensusBestTipUpdate,
    ConsensusBlockChainProofUpdate,
    ConsensusBlockReceived,
    ConsensusBlockSnarkVerifyPending,
    ConsensusBlockSnarkVerifySuccess,
    ConsensusDetectForkRange,
    ConsensusLongRangeForkResolve,
    ConsensusShortRangeForkResolve,
    EventSourceNewEvent,
    EventSourceProcessEvents,
    EventSourceWaitForEvents,
    EventSourceWaitTimeout,
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
    RpcSnarkPoolAvailableJobsGet,
    RpcSnarkerJobCommit,
    RpcSyncStatsGet,
    SnarkBlockVerifyError,
    SnarkBlockVerifyFinish,
    SnarkBlockVerifyInit,
    SnarkBlockVerifyPending,
    SnarkBlockVerifySuccess,
    SnarkPoolCheckTimeouts,
    SnarkPoolCommitmentCreate,
    SnarkPoolJobCommitmentAdd,
    SnarkPoolJobCommitmentTimeout,
    SnarkPoolJobsUpdate,
    SnarkPoolP2pSend,
    SnarkPoolP2pSendAll,
    TransitionFrontierSyncBestTipUpdate,
    TransitionFrontierSyncBlocksFetchSuccess,
    TransitionFrontierSyncBlocksNextApplyInit,
    TransitionFrontierSyncBlocksNextApplyPending,
    TransitionFrontierSyncBlocksNextApplySuccess,
    TransitionFrontierSyncBlocksPeerQueryError,
    TransitionFrontierSyncBlocksPeerQueryInit,
    TransitionFrontierSyncBlocksPeerQueryPending,
    TransitionFrontierSyncBlocksPeerQueryRetry,
    TransitionFrontierSyncBlocksPeerQuerySuccess,
    TransitionFrontierSyncBlocksPeersQuery,
    TransitionFrontierSyncBlocksPending,
    TransitionFrontierSyncBlocksSuccess,
    TransitionFrontierSyncInit,
    TransitionFrontierSyncLedgerInit,
    TransitionFrontierSyncLedgerRootPending,
    TransitionFrontierSyncLedgerRootSuccess,
    TransitionFrontierSyncLedgerSnarkedChildAccountsReceived,
    TransitionFrontierSyncLedgerSnarkedChildHashesReceived,
    TransitionFrontierSyncLedgerSnarkedPeerQueryError,
    TransitionFrontierSyncLedgerSnarkedPeerQueryInit,
    TransitionFrontierSyncLedgerSnarkedPeerQueryPending,
    TransitionFrontierSyncLedgerSnarkedPeerQueryRetry,
    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccess,
    TransitionFrontierSyncLedgerSnarkedPeersQuery,
    TransitionFrontierSyncLedgerSnarkedPending,
    TransitionFrontierSyncLedgerSnarkedSuccess,
    TransitionFrontierSyncLedgerStagedPartsFetchPending,
    TransitionFrontierSyncLedgerStagedPartsFetchSuccess,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchError,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchInit,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchPending,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccess,
    TransitionFrontierSyncLedgerStagedPartsPeerInvalid,
    TransitionFrontierSyncLedgerStagedPartsPeerValid,
    TransitionFrontierSyncLedgerStagedReconstructError,
    TransitionFrontierSyncLedgerStagedReconstructInit,
    TransitionFrontierSyncLedgerStagedReconstructPending,
    TransitionFrontierSyncLedgerStagedReconstructSuccess,
    TransitionFrontierSyncLedgerStagedSuccess,
    TransitionFrontierSyncLedgerSuccess,
    TransitionFrontierSynced,
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
    pub const COUNT: usize = 148;
}

impl ActionKindGet for Action {
    fn kind(&self) -> ActionKind {
        match self {
            Self::CheckTimeouts(a) => a.kind(),
            Self::EventSource(a) => a.kind(),
            Self::P2p(a) => a.kind(),
            Self::Snark(a) => a.kind(),
            Self::Consensus(a) => a.kind(),
            Self::TransitionFrontier(a) => a.kind(),
            Self::SnarkPool(a) => a.kind(),
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
            Self::BlockChainProofUpdate(a) => a.kind(),
            Self::BlockSnarkVerifyPending(a) => a.kind(),
            Self::BlockSnarkVerifySuccess(a) => a.kind(),
            Self::DetectForkRange(a) => a.kind(),
            Self::ShortRangeForkResolve(a) => a.kind(),
            Self::LongRangeForkResolve(a) => a.kind(),
            Self::BestTipUpdate(a) => a.kind(),
        }
    }
}

impl ActionKindGet for TransitionFrontierAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Sync(a) => a.kind(),
            Self::Synced(a) => a.kind(),
        }
    }
}

impl ActionKindGet for SnarkPoolAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::CommitmentCreate(a) => a.kind(),
            Self::CommitmentAdd(a) => a.kind(),
            Self::JobsUpdate(a) => a.kind(),
            Self::P2pSendAll(a) => a.kind(),
            Self::P2pSend(a) => a.kind(),
            Self::CheckTimeouts(a) => a.kind(),
            Self::JobCommitmentTimeout(a) => a.kind(),
        }
    }
}

impl ActionKindGet for RpcAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::GlobalStateGet(a) => a.kind(),
            Self::ActionStatsGet(a) => a.kind(),
            Self::SyncStatsGet(a) => a.kind(),
            Self::P2pConnectionOutgoingInit(a) => a.kind(),
            Self::P2pConnectionOutgoingPending(a) => a.kind(),
            Self::P2pConnectionOutgoingError(a) => a.kind(),
            Self::P2pConnectionOutgoingSuccess(a) => a.kind(),
            Self::P2pConnectionIncomingInit(a) => a.kind(),
            Self::P2pConnectionIncomingPending(a) => a.kind(),
            Self::P2pConnectionIncomingRespond(a) => a.kind(),
            Self::P2pConnectionIncomingError(a) => a.kind(),
            Self::P2pConnectionIncomingSuccess(a) => a.kind(),
            Self::SnarkPoolAvailableJobsGet(a) => a.kind(),
            Self::SnarkerJobCommit(a) => a.kind(),
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

impl ActionKindGet for ConsensusBlockChainProofUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBlockChainProofUpdate
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

impl ActionKindGet for ConsensusDetectForkRangeAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusDetectForkRange
    }
}

impl ActionKindGet for ConsensusShortRangeForkResolveAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusShortRangeForkResolve
    }
}

impl ActionKindGet for ConsensusLongRangeForkResolveAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusLongRangeForkResolve
    }
}

impl ActionKindGet for ConsensusBestTipUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::ConsensusBestTipUpdate
    }
}

impl ActionKindGet for TransitionFrontierSyncAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::BestTipUpdate(a) => a.kind(),
            Self::LedgerRootPending(a) => a.kind(),
            Self::LedgerRootSuccess(a) => a.kind(),
            Self::BlocksPending(a) => a.kind(),
            Self::BlocksPeersQuery(a) => a.kind(),
            Self::BlocksPeerQueryInit(a) => a.kind(),
            Self::BlocksPeerQueryRetry(a) => a.kind(),
            Self::BlocksPeerQueryPending(a) => a.kind(),
            Self::BlocksPeerQueryError(a) => a.kind(),
            Self::BlocksPeerQuerySuccess(a) => a.kind(),
            Self::BlocksFetchSuccess(a) => a.kind(),
            Self::BlocksNextApplyInit(a) => a.kind(),
            Self::BlocksNextApplyPending(a) => a.kind(),
            Self::BlocksNextApplySuccess(a) => a.kind(),
            Self::BlocksSuccess(a) => a.kind(),
            Self::Ledger(a) => a.kind(),
        }
    }
}

impl ActionKindGet for TransitionFrontierSyncedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSynced
    }
}

impl ActionKindGet for SnarkPoolCommitmentCreateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolCommitmentCreate
    }
}

impl ActionKindGet for SnarkPoolJobCommitmentAddAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolJobCommitmentAdd
    }
}

impl ActionKindGet for SnarkPoolJobsUpdateAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolJobsUpdate
    }
}

impl ActionKindGet for SnarkPoolP2pSendAllAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolP2pSendAll
    }
}

impl ActionKindGet for SnarkPoolP2pSendAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolP2pSend
    }
}

impl ActionKindGet for SnarkPoolCheckTimeoutsAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolCheckTimeouts
    }
}

impl ActionKindGet for SnarkPoolJobCommitmentTimeoutAction {
    fn kind(&self) -> ActionKind {
        ActionKind::SnarkPoolJobCommitmentTimeout
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

impl ActionKindGet for RpcSyncStatsGetAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcSyncStatsGet
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

impl ActionKindGet for RpcSnarkPoolAvailableJobsGetAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcSnarkPoolAvailableJobsGet
    }
}

impl ActionKindGet for RpcSnarkerJobCommitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcSnarkerJobCommit
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

impl ActionKindGet for TransitionFrontierSyncLedgerRootPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerRootPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerRootSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerRootSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPending
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeersQueryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeersQuery
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeerQueryInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeerQueryInit
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeerQueryRetryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeerQueryRetry
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeerQueryPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeerQueryPending
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeerQueryErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeerQueryError
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksPeerQuerySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksPeerQuerySuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksFetchSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksFetchSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksNextApplyInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksNextApplyInit
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksNextApplyPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksNextApplyPending
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksNextApplySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksNextApplySuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncBlocksSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncBlocksSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Init(a) => a.kind(),
            Self::Snarked(a) => a.kind(),
            Self::Staged(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
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

impl ActionKindGet for TransitionFrontierSyncLedgerInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::Pending(a) => a.kind(),
            Self::PeersQuery(a) => a.kind(),
            Self::PeerQueryInit(a) => a.kind(),
            Self::PeerQueryPending(a) => a.kind(),
            Self::PeerQueryRetry(a) => a.kind(),
            Self::PeerQueryError(a) => a.kind(),
            Self::PeerQuerySuccess(a) => a.kind(),
            Self::ChildHashesReceived(a) => a.kind(),
            Self::ChildAccountsReceived(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedAction {
    fn kind(&self) -> ActionKind {
        match self {
            Self::PartsFetchPending(a) => a.kind(),
            Self::PartsPeerFetchInit(a) => a.kind(),
            Self::PartsPeerFetchPending(a) => a.kind(),
            Self::PartsPeerFetchError(a) => a.kind(),
            Self::PartsPeerFetchSuccess(a) => a.kind(),
            Self::PartsPeerInvalid(a) => a.kind(),
            Self::PartsPeerValid(a) => a.kind(),
            Self::PartsFetchSuccess(a) => a.kind(),
            Self::ReconstructInit(a) => a.kind(),
            Self::ReconstructPending(a) => a.kind(),
            Self::ReconstructError(a) => a.kind(),
            Self::ReconstructSuccess(a) => a.kind(),
            Self::Success(a) => a.kind(),
        }
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeersQueryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeersQuery
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeerQueryInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeerQueryInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeerQueryPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeerQueryPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeerQueryRetryAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeerQueryRetry
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeerQueryError
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedPeerQuerySuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedChildHashesReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedChildHashesReceived
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedChildAccountsReceivedAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedChildAccountsReceived
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerSnarkedSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerSnarkedSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsFetchPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsFetchPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerFetchInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerFetchPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerFetchError
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerInvalid
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsPeerValidAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsPeerValid
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedPartsFetchSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedReconstructInitAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedReconstructInit
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedReconstructPendingAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedReconstructPending
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedReconstructErrorAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedReconstructError
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedReconstructSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedReconstructSuccess
    }
}

impl ActionKindGet for TransitionFrontierSyncLedgerStagedSuccessAction {
    fn kind(&self) -> ActionKind {
        ActionKind::TransitionFrontierSyncLedgerStagedSuccess
    }
}

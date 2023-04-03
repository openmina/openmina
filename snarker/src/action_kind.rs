use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use crate::event_source::{
    EventSourceAction, EventSourceNewEventAction, EventSourceProcessEventsAction,
    EventSourceWaitForEventsAction, EventSourceWaitTimeoutAction,
};
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
use crate::p2p::{P2pAction, P2pPeerReadyAction};
use crate::rpc::{
    RpcAction, RpcActionStatsGetAction, RpcFinishAction, RpcGlobalStateGetAction,
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingInitAction,
    RpcP2pConnectionIncomingPendingAction, RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingSuccessAction, RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingInitAction, RpcP2pConnectionOutgoingPendingAction,
    RpcP2pConnectionOutgoingSuccessAction,
};
use crate::{Action, ActionKindGet, CheckTimeoutsAction};

#[derive(
    Serialize, Deserialize, TryFromPrimitive, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy,
)]
#[repr(u16)]
pub enum ActionKind {
    None,
    CheckTimeouts,
    EventSourceNewEvent,
    EventSourceProcessEvents,
    EventSourceWaitForEvents,
    EventSourceWaitTimeout,
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
}

impl ActionKindGet for Action {
    fn kind(&self) -> ActionKind {
        match self {
            Self::CheckTimeouts(a) => a.kind(),
            Self::EventSource(a) => a.kind(),
            Self::P2p(a) => a.kind(),
            Self::Rpc(a) => a.kind(),
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
            Self::Finish(a) => a.kind(),
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

impl ActionKindGet for P2pPeerReadyAction {
    fn kind(&self) -> ActionKind {
        ActionKind::P2pPeerReady
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

impl ActionKindGet for RpcFinishAction {
    fn kind(&self) -> ActionKind {
        ActionKind::RpcFinish
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

use openmina_core::{
    impl_substate_access,
    log::{
        inner::{
            field::{display, DisplayValue},
            Value,
        },
        time_to_str, EventContext,
    },
    ActionEvent,
};
use p2p::{
    bootstrap::P2pNetworkKadBootstrapState,
    channels::{
        best_tip::P2pChannelsBestTipAction, best_tip_effectful::P2pChannelsBestTipEffectfulAction,
        rpc::P2pChannelsRpcAction, rpc_effectful::P2pChannelsRpcEffectfulAction,
        snark::P2pChannelsSnarkAction, snark_effectful::P2pChannelsSnarkEffectfulAction,
        snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
        snark_job_commitment_effectful::P2pChannelsSnarkJobCommitmentEffectfulAction,
        streaming_rpc::P2pChannelsStreamingRpcAction,
        streaming_rpc_effectful::P2pChannelsStreamingRpcEffectfulAction,
        transaction::P2pChannelsTransactionAction,
        transaction_effectful::P2pChannelsTransactionEffectfulAction,
    },
    connection::{
        incoming_effectful::P2pConnectionIncomingEffectfulAction,
        outgoing::P2pConnectionOutgoingAction,
        outgoing_effectful::P2pConnectionOutgoingEffectfulAction,
    },
    disconnection::P2pDisconnectionAction,
    disconnection_effectful::P2pDisconnectionEffectfulAction,
    identify::P2pIdentifyAction,
    network::identify::{
        stream_effectful::P2pNetworkIdentifyStreamEffectfulAction, P2pNetworkIdentifyState,
        P2pNetworkIdentifyStreamAction,
    },
    peer::P2pPeerAction,
    MioEvent, P2pAction, P2pEvent, P2pNetworkKadBootstrapAction, P2pNetworkKadRequestAction,
    P2pNetworkKademliaAction, P2pNetworkKademliaStreamAction, P2pNetworkSchedulerAction,
    P2pNetworkSchedulerEffectfulAction, P2pNetworkYamuxAction, P2pState, P2pStateTrait, PeerId,
};
use redux::{ActionMeta, EnablingCondition, SubStore};

use crate::service::ClusterService;

pub struct State(pub(crate) P2pState);
pub type Store = redux::Store<State, ClusterService, Action>;

impl State {
    pub fn state(&self) -> &P2pState {
        &self.0
    }
}

impl EnablingCondition<State> for Action {
    fn is_enabled(&self, state: &State, time: redux::Timestamp) -> bool {
        match self {
            Action::P2p(a) => a.is_enabled(&state.0, time),
            Action::Idle(a) => a.is_enabled(state, time),
        }
    }
}

impl SubStore<State, P2pState> for Store {
    type SubAction = P2pAction;

    type Service = ClusterService;

    fn state(&self) -> &P2pState {
        &self.state.get().0
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&P2pState, &mut Self::Service) {
        (&self.state.get().0, &mut self.service)
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<Self::SubAction> + redux::EnablingCondition<P2pState>,
    {
        self.sub_dispatch(action)
    }

    fn dispatch_callback<T>(&mut self, callback: redux::Callback<T>, args: T) -> bool
    where
        T: 'static,
        P2pAction: From<redux::AnyAction> + redux::EnablingCondition<P2pState>,
    {
        Store::dispatch_callback(self, callback, args)
    }
}

impl_substate_access!(State, P2pState, 0);

macro_rules! impl_p2p_state_access {
    ($state:ty, $substate_type:ty) => {
        impl openmina_core::SubstateAccess<$substate_type> for $state {
            fn substate(&self) -> openmina_core::SubstateResult<&$substate_type> {
                let substate: &P2pState = self.substate()?;
                substate.substate()
            }

            fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut $substate_type> {
                let substate: &mut P2pState = self.substate_mut()?;
                substate.substate_mut()
            }
        }
    };
}

impl_p2p_state_access!(State, P2pNetworkIdentifyState);
impl_p2p_state_access!(State, p2p::P2pNetworkState);
impl_p2p_state_access!(State, P2pNetworkKadBootstrapState);
impl_p2p_state_access!(State, p2p::P2pNetworkKadState);
impl_p2p_state_access!(State, p2p::P2pNetworkSchedulerState);
impl_p2p_state_access!(State, p2p::P2pLimits);
impl_p2p_state_access!(State, p2p::P2pNetworkPubsubState);
impl_p2p_state_access!(State, p2p::P2pConfig);

impl P2pStateTrait for State {}

#[derive(Debug, derive_more::From)]
pub enum Action {
    P2p(P2pAction),
    Idle(IdleAction),
}

impl From<redux::AnyAction> for Action {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}

#[derive(Debug)]
pub struct IdleAction;

impl EnablingCondition<State> for IdleAction {
    fn is_enabled(&self, _state: &State, _time: redux::Timestamp) -> bool {
        true
    }
}

struct ActionLoggerContext {
    time: redux::Timestamp,
    time_str: String,
    node_id: DisplayValue<PeerId>,
}

impl ActionLoggerContext {
    fn new(time: redux::Timestamp, node_id: PeerId) -> Self {
        ActionLoggerContext {
            time,
            time_str: time_to_str(time),
            node_id: display(node_id),
        }
    }
}

impl EventContext for ActionLoggerContext {
    fn timestamp(&self) -> redux::Timestamp {
        self.time
    }

    fn time(&self) -> &'_ dyn Value {
        &self.time_str
    }

    fn node_id(&self) -> &'_ dyn Value {
        &self.node_id
    }
}

pub(super) fn log_action(action: &Action, meta: &ActionMeta, node_id: PeerId) {
    if let Action::P2p(action) = action {
        ActionEvent::action_event(action, &ActionLoggerContext::new(meta.time(), node_id));
    }
}

pub(super) fn event_effect(store: &mut crate::redux::Store, event: P2pEvent) -> bool {
    match event {
        P2pEvent::MioEvent(event) => match event {
            MioEvent::InterfaceDetected(ip) => {
                SubStore::dispatch(store, P2pNetworkSchedulerAction::InterfaceDetected { ip })
            }
            MioEvent::InterfaceExpired(ip) => {
                SubStore::dispatch(store, P2pNetworkSchedulerAction::InterfaceExpired { ip })
            }
            MioEvent::ListenerReady { listener } => {
                SubStore::dispatch(store, P2pNetworkSchedulerAction::ListenerReady { listener })
            }
            MioEvent::ListenerError { listener, error } => SubStore::dispatch(
                store,
                P2pNetworkSchedulerAction::ListenerError { listener, error },
            ),
            MioEvent::IncomingConnectionIsReady { listener } => SubStore::dispatch(
                store,
                P2pNetworkSchedulerEffectfulAction::IncomingConnectionIsReady { listener },
            ),
            MioEvent::IncomingConnectionDidAccept(addr, result) => SubStore::dispatch(
                store,
                P2pNetworkSchedulerAction::IncomingDidAccept { addr, result },
            ),
            MioEvent::OutgoingConnectionDidConnect(addr, result) => SubStore::dispatch(
                store,
                P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result },
            ),
            MioEvent::IncomingDataIsReady(addr) => SubStore::dispatch(
                store,
                P2pNetworkSchedulerAction::IncomingDataIsReady { addr },
            ),
            MioEvent::IncomingDataDidReceive(addr, result) => SubStore::dispatch(
                store,
                P2pNetworkSchedulerAction::IncomingDataDidReceive {
                    addr,
                    result: result.map(From::from),
                },
            ),
            MioEvent::OutgoingDataDidSend(_, _result) => true,
            MioEvent::ConnectionDidClose(addr, result) => {
                if let Err(e) = result {
                    SubStore::dispatch(
                        store,
                        P2pNetworkSchedulerAction::Error {
                            addr,
                            error: p2p::P2pNetworkConnectionError::MioError(e),
                        },
                    )
                } else {
                    SubStore::dispatch(
                        store,
                        P2pNetworkSchedulerAction::Error {
                            addr,
                            error: p2p::P2pNetworkConnectionError::RemoteClosed,
                        },
                    )
                }
            }
            MioEvent::ConnectionDidCloseOnDemand(addr) => {
                SubStore::dispatch(store, P2pNetworkSchedulerAction::Prune { addr })
            }
        },
        _ => false,
    }
}

macro_rules! impl_from_p2p {
    ($sub_action:ty) => {
        impl From<$sub_action> for Action {
            fn from(value: $sub_action) -> Self {
                Self::P2p(P2pAction::from(value))
            }
        }
    };
}

impl_from_p2p!(P2pNetworkKademliaAction);
impl_from_p2p!(P2pNetworkKademliaStreamAction);
impl_from_p2p!(P2pNetworkKadRequestAction);
impl_from_p2p!(P2pNetworkKadBootstrapAction);
impl_from_p2p!(P2pPeerAction);
impl_from_p2p!(P2pNetworkYamuxAction);
impl_from_p2p!(P2pConnectionOutgoingAction);
impl_from_p2p!(P2pNetworkSchedulerAction);
impl_from_p2p!(P2pNetworkIdentifyStreamAction);
impl_from_p2p!(P2pIdentifyAction);
impl_from_p2p!(P2pNetworkIdentifyStreamEffectfulAction);
impl_from_p2p!(p2p::P2pNetworkSelectAction);
impl_from_p2p!(p2p::P2pNetworkPnetAction);
impl_from_p2p!(p2p::P2pNetworkNoiseAction);
impl_from_p2p!(p2p::connection::incoming::P2pConnectionIncomingAction);
impl_from_p2p!(p2p::P2pNetworkPubsubAction);
impl_from_p2p!(p2p::P2pNetworkPubsubEffectfulAction);
impl_from_p2p!(P2pChannelsTransactionAction);
impl_from_p2p!(P2pChannelsSnarkAction);
impl_from_p2p!(p2p::P2pNetworkRpcAction);
impl_from_p2p!(P2pChannelsRpcAction);
impl_from_p2p!(P2pDisconnectionAction);
impl_from_p2p!(p2p::P2pNetworkSchedulerEffectfulAction);
impl_from_p2p!(p2p::P2pNetworkPnetEffectfulAction);
impl_from_p2p!(P2pChannelsBestTipAction);
impl_from_p2p!(P2pChannelsSnarkJobCommitmentAction);
impl_from_p2p!(P2pChannelsStreamingRpcAction);
impl_from_p2p!(P2pConnectionIncomingEffectfulAction);
impl_from_p2p!(P2pConnectionOutgoingEffectfulAction);
impl_from_p2p!(P2pDisconnectionEffectfulAction);
impl_from_p2p!(P2pChannelsBestTipEffectfulAction);
impl_from_p2p!(P2pChannelsStreamingRpcEffectfulAction);
impl_from_p2p!(P2pChannelsTransactionEffectfulAction);
impl_from_p2p!(P2pChannelsSnarkJobCommitmentEffectfulAction);
impl_from_p2p!(P2pChannelsRpcEffectfulAction);
impl_from_p2p!(P2pChannelsSnarkEffectfulAction);

impl p2p::P2pActionTrait<State> for Action {}

use openmina_core::{
    log::{
        inner::{
            field::{display, DisplayValue},
            Value,
        },
        time_to_str, EventContext,
    },
    ActionEvent, SubstateAccess, SubstateResult,
};
use p2p::{MioEvent, P2pAction, P2pEvent, P2pNetworkSchedulerAction, P2pState, PeerId};
use redux::{ActionMeta, EnablingCondition, SubStore};

use crate::service::ClusterService;

pub(crate) struct State(pub(crate) P2pState);

pub(crate) type Store = redux::Store<State, ClusterService, Action>;

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

impl SubstateAccess<P2pState> for State {
    fn substate(&self) -> SubstateResult<&P2pState> {
        Ok(&self.0)
    }
    fn substate_mut(&mut self) -> SubstateResult<&mut P2pState> {
        Ok(&mut self.0)
    }
}

#[derive(Debug, derive_more::From)]
pub(crate) enum Action {
    P2p(P2pAction),
    Idle(IdleAction),
}

impl From<redux::AnyAction> for Action {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}

#[derive(Debug)]
pub(crate) struct IdleAction;

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
                P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener },
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
        },
        _ => false,
    }
}

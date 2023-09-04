use serde::{Deserialize, Serialize};

pub type EventSourceActionWithMeta = redux::ActionWithMeta<EventSourceAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum EventSourceAction {
    ProcessEvents(EventSourceProcessEventsAction),
    NewEvent(EventSourceNewEventAction),
    WaitForEvents(EventSourceWaitForEventsAction),
    WaitTimeout(EventSourceWaitTimeoutAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceProcessEventsAction {}

impl redux::EnablingCondition<crate::State> for EventSourceProcessEventsAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceNewEventAction {
    pub event: super::Event,
}

impl redux::EnablingCondition<crate::State> for EventSourceNewEventAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

/// Next action won't be dispatched, until new events are available out_dir
/// wait times out.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceWaitForEventsAction {}

impl redux::EnablingCondition<crate::State> for EventSourceWaitForEventsAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

/// Waiting for events has timed out.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceWaitTimeoutAction {}

impl redux::EnablingCondition<crate::State> for EventSourceWaitTimeoutAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

impl From<EventSourceProcessEventsAction> for crate::Action {
    fn from(a: EventSourceProcessEventsAction) -> Self {
        Self::EventSource(EventSourceAction::ProcessEvents(a))
    }
}

impl From<EventSourceNewEventAction> for crate::Action {
    fn from(a: EventSourceNewEventAction) -> Self {
        Self::EventSource(EventSourceAction::NewEvent(a))
    }
}

impl From<EventSourceWaitForEventsAction> for crate::Action {
    fn from(a: EventSourceWaitForEventsAction) -> Self {
        Self::EventSource(EventSourceAction::WaitForEvents(a))
    }
}

impl From<EventSourceWaitTimeoutAction> for crate::Action {
    fn from(a: EventSourceWaitTimeoutAction) -> Self {
        Self::EventSource(EventSourceAction::WaitTimeout(a))
    }
}

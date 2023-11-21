use serde::{Deserialize, Serialize};

pub type EventSourceActionWithMeta = redux::ActionWithMeta<EventSourceAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum EventSourceAction {
    ProcessEvents(EventSourceProcessEventsAction),
    NewEvent(EventSourceNewEventAction),
    WaitForEvents(EventSourceWaitForEventsAction),
    WaitTimeout(EventSourceWaitTimeoutAction),
}

/// Notify state machine that the new events might be received/available,
/// so trigger processing of those events.
/// These are processed in batches of up to 1024 events.
///
/// This action will be continously triggered, until there are no more
/// events in the queue, in which case `EventSourceWaitForEventsAction`
/// will be dispatched.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceProcessEventsAction {}

impl redux::EnablingCondition<crate::State> for EventSourceProcessEventsAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

/// Process newly retrieved event.
/// Events can come from: P2P connections, snark work, RPCs,
/// and communication with the snark worker process. 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSourceNewEventAction {
    pub event: super::Event,
}

impl redux::EnablingCondition<crate::State> for EventSourceNewEventAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

/// Next action won't be dispatched, until new events are available or
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

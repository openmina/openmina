use serde::{Deserialize, Serialize};

pub type EventSourceActionWithMeta = redux::ActionWithMeta<EventSourceAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventSourceAction {
    /// Notify state machine that the new events might be received/available,
    /// so trigger processing of those events.
    ///
    /// This action will be continuously triggered, until there are no more
    /// events in the queue, in which case `EventSourceWaitForEventsAction`
    /// will be dispatched.
    ProcessEvents,

    /// Process newly retrieved event.
    NewEvent { event: super::Event },

    /// Next action won't be dispatched, until new events are available or
    /// wait times out.
    WaitForEvents,

    /// Waiting for events has timed out.
    WaitTimeout,
}

impl redux::EnablingCondition<crate::State> for EventSourceAction {
    fn is_enabled(&self, _: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            EventSourceAction::ProcessEvents => true,
            EventSourceAction::NewEvent { event: _ } => true,
            EventSourceAction::WaitForEvents => true,
            EventSourceAction::WaitTimeout => true,
        }
    }
}

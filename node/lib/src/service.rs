pub use redux::TimeService;

pub use crate::event_source::EventSourceService;

pub trait Service: TimeService + EventSourceService {}

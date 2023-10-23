//! Event source state machine.
//!
//! Here is the logic for retrieving and processing events. This is the
//! main driver of state machine.
//!
//! From outside the state machine, only dispatched actions should be
//! either event source actions or special `CheckTimeoutsAction`.

mod event;
pub use event::*;

mod event_source_actions;
pub use event_source_actions::*;

mod event_source_effects;
pub use event_source_effects::*;

mod event_source_service;
pub use event_source_service::*;

/// Must only be used in logging and even there it's not prefferable.
///
/// This **MUST** only be used in places which doesn't have access to any
/// of the following: `redux::Store`, global state where time is stored,
/// `redux::ActionMeta::time()`.
pub fn system_time() -> redux::Timestamp {
    redux::Timestamp::global_now()
}

pub fn time_to_str(t: redux::Timestamp) -> String {
    let t = u64::from(t);
    t.to_string()
}

pub mod inner {
    pub use tracing::*;
}

#[macro_export]
macro_rules! log_entry {
    ($level:ident, $time:expr; $($tts:tt)*) => {
        $crate::log::inner::$level!(time = $crate::log::time_to_str($time), $($tts)*);
    };
    ($level:ident; $($tts:tt)*) => {
        $crate::log::inner::$level!(time = $crate::log::time_to_str($crate::log::system_time()), $($tts)*);
    };
}

#[macro_export]
macro_rules! trace {
    ($time:expr; $($tts:tt)*) => {
        $crate::log_entry!(trace, $time; $($tts)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($time:expr; $($tts:tt)*) => {
        $crate::log_entry!(debug, $time; $($tts)*);
    };
}

#[macro_export]
macro_rules! info {
    ($time:expr; $($tts:tt)*) => {
        $crate::log_entry!(info, $time; $($tts)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($time:expr; $($tts:tt)*) => {
        $crate::log_entry!(warn, $time; $($tts)*);
    };
}

#[macro_export]
macro_rules! error {
    ($time:expr; $($tts:tt)*) => {
        $crate::log_entry!(error, $time; $($tts)*);
    };
}

pub use crate::{debug, error, info, trace, warn};

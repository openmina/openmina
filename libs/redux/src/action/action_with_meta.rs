use std::time::Duration;

use crate::{SystemTime, Timestamp};

use super::{ActionId, ActionMeta, RecursionDepth};

/// Action with additional metadata like: id.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionWithMeta<Action> {
    pub(super) meta: ActionMeta,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub(super) action: Action,
}

impl<Action> ActionWithMeta<Action> {
    pub fn meta(&self) -> &ActionMeta {
        &self.meta
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    #[inline(always)]
    pub fn id(&self) -> ActionId {
        self.meta.id()
    }

    /// Recursion depth of a given action.
    #[inline(always)]
    pub fn depth(&self) -> RecursionDepth {
        self.meta.depth()
    }

    #[inline(always)]
    pub fn time(&self) -> Timestamp {
        self.meta.time()
    }

    #[inline(always)]
    pub fn sys_time(&self) -> SystemTime {
        self.meta.sys_time()
    }

    #[inline(always)]
    pub fn time_as_nanos(&self) -> u64 {
        self.meta.time_as_nanos()
    }

    #[inline(always)]
    pub fn duration_since_epoch(&self) -> Duration {
        self.meta.duration_since_epoch()
    }

    #[inline(always)]
    pub fn duration_since(&self, other: &ActionWithMeta<Action>) -> Duration {
        self.meta.duration_since(&other.meta)
    }

    /// Splits the struct into a tuple of action and it's metadata.
    #[inline(always)]
    pub fn split(self) -> (Action, ActionMeta) {
        (self.action, self.meta)
    }
}

use std::time::Duration;

use crate::Timestamp;

/// Time in nanoseconds from [std::time::UNIX_EPOCH].
///
/// Each action will have unique id. If two actions happen at the same time,
/// id must be increased by 1 for second action, to ensure uniqueness of id.
///
/// u64 is enough to contain time in nanoseconds at most 584 years
/// after `UNIX_EPOCH` (1970-01-01 00:00:00 UTC).
///
/// ```
/// //           nano     micro  milli  sec    min  hour day  year
/// assert_eq!(u64::MAX / 1000 / 1000 / 1000 / 60 / 60 / 24 / 365, 584);
/// ```
#[cfg_attr(feature = "fuzzing", derive(fuzzcheck::DefaultMutator))]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionId(Timestamp);

impl ActionId {
    pub const ZERO: Self = Self(Timestamp::ZERO);

    /// Caller must make sure such action actually exists!
    #[inline(always)]
    pub fn new_unchecked(value: u64) -> Self {
        Self(Timestamp::new(value))
    }

    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn next(&self, time_passed: u64) -> Self {
        Self(self.0 + time_passed.max(1))
    }

    pub fn duration_since(&self, other: ActionId) -> Duration {
        let d = self.0.checked_sub(other.0);
        debug_assert!(d.is_some());
        d.unwrap_or(Duration::ZERO)
    }
}

impl From<ActionId> for Timestamp {
    #[inline(always)]
    fn from(id: ActionId) -> Self {
        id.0
    }
}

impl From<ActionId> for u64 {
    #[inline(always)]
    fn from(id: ActionId) -> Self {
        id.0.into()
    }
}

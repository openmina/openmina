pub use crate::instant::Instant;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::SystemTime;

#[cfg(target_arch = "wasm32")]
pub use wasm_timer::SystemTime;

/// Time in nanoseconds from [std::time::UNIX_EPOCH].
///
/// Used as an ID+Timestamp for an Action`.
/// Each action will have an unique id. If two actions happen at the same time,
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
pub struct Timestamp(u64);

impl Timestamp {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub fn new(nanos_from_unix_epoch: u64) -> Self {
        Self(nanos_from_unix_epoch)
    }

    #[inline(always)]
    pub fn global_now() -> Self {
        Self::new(crate::monotonic_to_time(None))
    }

    pub fn checked_sub(self, rhs: Timestamp) -> Option<Duration> {
        self.0.checked_sub(rhs.0).map(Duration::from_nanos)
    }

    pub fn checked_add(self, other: u64) -> Option<Timestamp> {
        self.0.checked_add(other).map(Timestamp)
    }
}

impl From<Timestamp> for u64 {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}

impl From<Timestamp> for SystemTime {
    fn from(value: Timestamp) -> Self {
        Self::UNIX_EPOCH + Duration::from_nanos(value.into())
    }
}

impl std::ops::Add for Timestamp {
    type Output = Timestamp;
    #[inline]
    fn add(self, other: Timestamp) -> Timestamp {
        Timestamp(self.0 + other.0)
    }
}

impl std::ops::Add<u64> for Timestamp {
    type Output = Timestamp;
    #[inline]
    fn add(self, other: u64) -> Timestamp {
        Timestamp(self.0 + other)
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Timestamp;
    #[inline]
    fn add(self, other: Duration) -> Timestamp {
        Timestamp(self.0 + other.as_nanos() as u64)
    }
}

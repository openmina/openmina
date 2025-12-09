use crate::Instant;

pub trait Service: TimeService {}

/// Time service.
pub trait TimeService {
    fn monotonic_time(&mut self) -> Instant {
        Instant::now()
    }
}

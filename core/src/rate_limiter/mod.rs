use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests_per_duration: u64,
    instant: Instant,
    requests: u64,
}

impl RateLimiter {
    /// TODO: Maybe move this to value inside struct so it can be changed, while running
    const DURATION: Duration = Duration::from_secs(1);

    pub fn new(requests_per_duration: u64) -> RateLimiter {
        RateLimiter {
            requests_per_duration,
            instant: Instant::now(),
            requests: 0,
        }
    }

    pub fn check(&mut self) -> bool {
        let can_run = self.check_requests() || self.check_time();
        if can_run {
            self.requests += 1;
        }

        can_run
    }

    fn check_requests(&mut self) -> bool {
        self.requests < self.requests_per_duration
    }

    fn check_time(&mut self) -> bool {
        let now = Instant::now();
        let bench = now - Self::DURATION;
        let reclaimed = self.instant < bench;

        if reclaimed {
            self.instant = now;
            self.requests = 0;
        }

        reclaimed
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::RateLimiter;

    #[bench]
    /// Benchmark the rate limiter to see how long does it take to execute
    /// result is about 14ns per iter
    fn bench_rate_limiter(bencher: &mut Bencher) {
        let mut rate_limiter = RateLimiter::new(10);

        bencher.iter(|| rate_limiter.check());
    }

    #[test]
    /// Check the rate limit and that it works correctly
    fn test_rate_limiter() {
        let rate_limit = 10;
        let mut rate_limiter = RateLimiter::new(rate_limit);

        for _ in 0..rate_limit {
            assert!(rate_limiter.check());
        }

        for _ in 0..10 {
            assert!(!rate_limiter.check());
        }
    }
}

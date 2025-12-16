#[cfg(not(target_arch = "wasm32"))]
pub type Instant = std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub type Instant = wasm_instant::Instant;

#[cfg(target_arch = "wasm32")]
pub mod wasm_instant {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(inline_js = r#"
export function performance_now() {
  return performance.now();
}"#)]
    extern "C" {
        fn performance_now() -> f64;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(u64);

    impl Instant {
        pub fn now() -> Self {
            Self((performance_now() * 1000.0) as u64)
        }
        pub fn duration_since(&self, earlier: Instant) -> Duration {
            Duration::from_micros(self.0 - earlier.0)
        }
        pub fn elapsed(&self) -> Duration {
            Self::now().duration_since(*self)
        }
        pub fn checked_add(&self, duration: Duration) -> Option<Self> {
            match duration.as_micros().try_into() {
                Ok(duration) => self.0.checked_add(duration).map(|i| Self(i)),
                Err(_) => None,
            }
        }
        pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
            match duration.as_micros().try_into() {
                Ok(duration) => self.0.checked_sub(duration).map(|i| Self(i)),
                Err(_) => None,
            }
        }
    }

    use std::{ops::*, time::Duration};

    impl Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, other: Duration) -> Instant {
            self.checked_add(other).unwrap()
        }
    }
    impl Sub<Duration> for Instant {
        type Output = Instant;
        fn sub(self, other: Duration) -> Instant {
            self.checked_sub(other).unwrap()
        }
    }
    impl Sub<Instant> for Instant {
        type Output = Duration;
        fn sub(self, other: Instant) -> Duration {
            self.duration_since(other)
        }
    }
    impl AddAssign<Duration> for Instant {
        fn add_assign(&mut self, other: Duration) {
            *self = *self + other;
        }
    }
    impl SubAssign<Duration> for Instant {
        fn sub_assign(&mut self, other: Duration) {
            *self = *self - other;
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    n: f64,
    mean: f64,
    q: f64,
}

#[derive(Debug, Default, Display, Serialize, Deserialize)]
#[display(fmt = "mean = {mean}μs")]
pub struct PerfStats {
    pub mean: u64,
}

impl From<Stats> for PerfStats {
    fn from(source: Stats) -> Self {
        let mean = Duration::from_secs_f64(source.mean)
            .as_micros()
            .try_into()
            .unwrap();
        PerfStats { mean }
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            todo!()
        } else {
            write!(f, "mean: {}, stdev: {}", self.mean, self.q / self.n)
        }
    }
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dur: Duration) {
        let x = dur.as_secs_f64();
        self.n += 1.;
        let p = x - self.mean;
        self.mean += p / self.n;
        self.q += p * (x - self.mean);
    }
}

const ITERS: (usize, usize) = (20, 5);

pub fn bench<O, F: Fn() -> O>(f: F) -> PerfStats {
    bench_internal(ITERS.0, ITERS.1, &f)
}

pub fn bench_iters<O, F: Fn() -> O>(iters: usize, f: F) -> PerfStats {
    bench_internal(iters, iters / 10, &f)
}

fn bench_internal<O, F: Fn() -> O>(iters: usize, wu_cd_iters: usize, f: &F) -> PerfStats {
    assert!(iters >= 20, "Number of iterations is too low");
    assert!(iters / wu_cd_iters > 3, "Warm-up/cool-down is too long");
    let mut stats = Stats::new();
    for i in 0..iters {
        let time = duration_of(f);
        if i >= wu_cd_iters && i < iters - wu_cd_iters {
            stats.update(time);
        }
    }
    stats.into()
}

use std::time::Duration;

use derive_more::Display;
use serde::{Deserialize, Serialize};

pub fn duration_of<F: Fn() -> O, O>(f: F) -> Duration {
    let then = Instant::now();
    let _ = f();
    Instant::now() - then
}

use super::measure::PerfStats;

pub fn perf_benchmark<F: Fn() -> O, O>(id: &str, f: F) -> PerfStats {
    let stats = super::measure::bench(f);
    log!("\nperformance stats for `{id}`:\n{stats}");
    stats
}

pub fn perf_log_toml<F: Fn() -> O, O>(id: &str, f: F) -> PerfStats {
    let stats = super::measure::bench(f);
    log!("\nperformance stats for `{id}`:\n{stats}", stats = toml::to_string(&stats).unwrap());
    stats
}

#[macro_export]
macro_rules! perf_bench {
    ($test:ident, $thresh:expr) => {
        $crate::threshold::check_threshold_with_args(
            || $crate::perf::benchmark::perf_benchmark(stringify!($test), $test),
            "perf_bench",
            stringify!($test),
            $thresh,
        )
    };
}

#[macro_export]
macro_rules! perf_bench_cmp_with_toml {
    ($test:ident $(,)?) => {{
        let value = $crate::perf::benchmark::perf_log_toml(stringify!($test), $test);
        Result::<
            $crate::perf::measure::PerfStats,
            $crate::threshold::CheckThresholdError<$crate::perf::compare::PerfThresholdsError>,
        >::Ok(value)
    }};
    ($test:ident, $toml:expr, $limits:expr $(,)?) => {{
        $crate::threshold::check_threshold_with_str(
            || $crate::perf::benchmark::perf_benchmark(stringify!($test), $test),
            $toml,
            $limits,
        )
    }};
}

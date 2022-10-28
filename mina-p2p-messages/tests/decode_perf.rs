mod decode_bench;
use decode_bench::*;

use alloc_test::{
    perf::compare::{PerfThresholds, PerfThresholdsBuilder, PerfThresholdsError},
    perf_bench,
    threshold::{CheckThresholdError, Threshold},
};

#[cfg(target_family = "wasm")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn perf_limits() -> PerfThresholds {
    let r = Threshold::ratio(1, 5);
    PerfThresholdsBuilder::default().mean(r).build().unwrap()
}

#[test]
#[ignore = "Memory allocation benchmark test should be run individually"]
fn decode_perf() -> Result<(), CheckThresholdError<PerfThresholdsError>> {
    let limits = perf_limits();
    perf_bench!(snark_pool_diff, &limits)?;
    perf_bench!(new_state, &limits)?;
    perf_bench!(staged_ledger, &limits)?;
    perf_bench!(incoming_rpc, &limits)?;
    Ok(())
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn decode_perf_wasm() -> Result<(), CheckThresholdError<PerfThresholdsError>> {
    use alloc_test::perf_bench_cmp_with_toml;
    let limits = perf_limits();
    perf_bench_cmp_with_toml!(
        snark_pool_diff,
        include_str!("decode-bench/perf/snark_pool_diff.wasm.toml"),
        &limits,
    )?;
    perf_bench_cmp_with_toml!(
        new_state,
        include_str!("decode-bench/perf/new_state.wasm.toml"),
        &limits,
    )?;
    perf_bench_cmp_with_toml!(
        staged_ledger,
        include_str!("decode-bench/perf/staged_ledger.wasm.toml"),
        &limits,
    )?;
    perf_bench_cmp_with_toml!(
        incoming_rpc,
        include_str!("decode-bench/perf/incoming_rpc.wasm.toml"),
        &limits,
    )?;
    Ok(())
}

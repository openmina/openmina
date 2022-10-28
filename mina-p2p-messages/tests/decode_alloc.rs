mod decode_bench;
use decode_bench::*;

use alloc_test::alloc::measure::MemoryTracingHooks;
use alloc_test::{
    alloc::{
        compare::{AllocThresholds, AllocThresholdsBuilder, AllocThresholdsError},
    },
    alloc_bench,
    threshold::{CheckThresholdError, Threshold},
};

#[global_allocator]
static ALLOCATOR: alloc_test::alloc::allocator::TracingAllocator<
    MemoryTracingHooks,
    std::alloc::System,
> = alloc_test::alloc::default_tracing_allocator();

#[cfg(target_family = "wasm")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn limits() -> AllocThresholds {
    let r = Threshold::ratio(1, 20);
    AllocThresholdsBuilder::default()
        .peak(r)
        .total_size(r)
        .total_num(r)
        .build()
        .unwrap()
}

#[test]
#[ignore = "Memory allocation benchmark test should be run individually"]
fn decode_alloc() -> Result<(), CheckThresholdError<AllocThresholdsError>> {
    let limits = limits();
    alloc_bench!(tx_pool_diff, &limits)?;
    alloc_bench!(snark_pool_diff, &limits)?;
    alloc_bench!(new_state, &limits)?;
    alloc_bench!(staged_ledger, &limits)?;
    alloc_bench!(incoming_rpc, &limits)?;
    Ok(())
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn decode_alloc_wasm() -> Result<(), CheckThresholdError<AllocThresholdsError>> {
    use alloc_test::alloc_bench_cmp_with_toml;
    let limits = limits();
    alloc_bench_cmp_with_toml!(
        tx_pool_diff,
        // include_str!("decode-bench/alloc/tx_pool_diff.wasm.toml"),
        // &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        snark_pool_diff,
        // include_str!("decode-bench/alloc/snark_pool_diff.wasm.toml"),
        // &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        new_state,
        // include_str!("decode-bench/alloc/new_state.wasm.toml"),
        // &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        staged_ledger,
        include_str!("decode-bench/alloc/staged_ledger.wasm.toml"),
        &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        incoming_rpc,
        include_str!("decode-bench/alloc/incoming_rpc.wasm.toml"),
        &limits,
    )?;
    Ok(())
}

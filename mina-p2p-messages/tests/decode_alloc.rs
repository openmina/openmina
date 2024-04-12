mod decode_bench;
use std::collections::BTreeMap;
use std::fs;

use binprot::BinProtWrite;
use decode_bench::*;

use alloc_test::alloc::measure::MemoryTracingHooks;
use alloc_test::{
    alloc::compare::{AllocThresholds, AllocThresholdsBuilder, AllocThresholdsError},
    alloc_bench,
    threshold::{CheckThresholdError, Threshold},
};
use mina_p2p_messages::rpc_kernel::Message;
use mina_p2p_messages::utils::{FromBinProtStream, Greedy};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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
        include_str!("decode-bench/alloc/tx_pool_diff.wasm.toml"),
        &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        snark_pool_diff,
        include_str!("decode-bench/alloc/snark_pool_diff.wasm.toml"),
        &limits,
    )?;
    alloc_bench_cmp_with_toml!(
        new_state,
        include_str!("decode-bench/alloc/new_state.wasm.toml"),
        &limits,
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

#[ignore = "need to fix bin files in `v2/gossip`"]
#[test]
fn decode_alloc_check_inputs() {
    tx_pool_diff();
    snark_pool_diff();
    new_state();
    //staged_ledger();
    //incoming_rpc();
}

mod utils;

#[test]
#[ignore = "Utility to combine catchup RPC messages into a sequence"]
fn collect_incoming_rpcs() {
    let mut queries = BTreeMap::new();
    utils::for_all("v2/rpc/catchup-messages/out", |path, mut bytes| {
        let modified = fs::metadata(path).unwrap().modified().unwrap();
        while !bytes.is_empty() {
            if let Some(b) =
                bytes.strip_prefix(b"\x07\x00\x00\x00\x00\x00\x00\x00\x02\xfd\x52\x50\x43\x00\x01")
            {
                bytes = b;
                continue;
            }
            if let Message::Query(q) = Message::<Greedy>::read_from_stream(&mut bytes).unwrap() {
                let key = q.id;
                queries.insert(key, (modified, q));
            }
        }
    })
    .unwrap();

    let mut pairs = Vec::new();
    utils::for_all("v2/rpc/catchup-messages/in", |path, mut bytes| {
        let modified = fs::metadata(path).unwrap().modified().unwrap();
        while !bytes.is_empty() {
            if let Some(b) =
                bytes.strip_prefix(b"\x07\x00\x00\x00\x00\x00\x00\x00\x02\xfd\x52\x50\x43\x00\x01")
            {
                bytes = b;
                continue;
            }
            if let Message::Response(r) = Message::<Greedy>::read_from_stream(&mut bytes).unwrap() {
                let (qm, q) = queries.remove(&r.id).unwrap();
                pairs.push((qm, q, modified, r));
            }
        }
    })
    .unwrap();

    pairs.sort_by_key(|(qm, _, _, _)| *qm);

    let mut out = fs::File::create(utils::files_path("v2/rpc/catchup_in1.bin").unwrap()).unwrap();
    for (qm, q, rm, r) in pairs {
        println!(
            "{}, duration is {:?}, queried at {} and reply received at {}",
            q.tag.to_string_lossy(),
            rm.duration_since(qm).unwrap(),
            OffsetDateTime::from(qm).format(&Rfc3339).unwrap(),
            OffsetDateTime::from(rm).format(&Rfc3339).unwrap(),
        );
        (q.tag, q.version).binprot_write(&mut out).unwrap();
        Message::Response(r).binprot_write(&mut out).unwrap();
    }
}

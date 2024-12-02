use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ledger::AccountIndex;
use mina_p2p_messages::v2::EpochSeed;
use num::BigInt;
use openmina_node_account::AccountSecretKey;
use std::str::FromStr;
use vrf::{evaluate_vrf, keypair_from_bs58_string, VrfEvaluationInput};

fn benchmark_evaluate_vrf(c: &mut Criterion) {
    let vrf_input = VrfEvaluationInput::new(
        keypair_from_bs58_string("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9"),
        EpochSeed::from_str("2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA").unwrap(),
        AccountSecretKey::genesis_producer().public_key(),
        6,
        AccountIndex(2),
        BigInt::from_str("1000000000000000").expect("Cannot convert to BigInt"),
        BigInt::from_str("6000000000001000").expect("Cannot convert to BigInt"),
    );

    c.bench_function("evaluate_vrf", |b| {
        b.iter(|| {
            black_box(evaluate_vrf(vrf_input.clone()).expect("Failed to evaluate vrf"));
        })
    });
}

criterion_group!(benches, benchmark_evaluate_vrf);
criterion_main!(benches);

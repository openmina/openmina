//! Database benchmarks
//!
//! Run with:
//! ```sh
//! cargo bench --bench database
//! ```
//!
//! Or from the workspace root:
//! ```sh
//! cd ledger && cargo bench --bench database
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mina_tree::*;

fn benchmark_account_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("account_generation");
    group.sample_size(10);

    for naccounts in [1_000, 10_000, 120_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(naccounts),
            &naccounts,
            |b, &naccounts| {
                b.iter(|| {
                    let mut db = Database::<V2>::create(20);
                    let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

                    for (index, mut account) in accounts.into_iter().enumerate() {
                        account.token_id = TokenId::from(index as u64);
                        let id = account.id();
                        db.get_or_create_account(id, account).unwrap();
                    }

                    black_box(db)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_merkle_root_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_root_computation");
    group.sample_size(10);

    for naccounts in [1_000, 10_000, 120_000] {
        // Prepare the database with accounts
        let mut db = Database::<V2>::create(20);
        let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

        for (index, mut account) in accounts.into_iter().enumerate() {
            account.token_id = TokenId::from(index as u64);
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(naccounts),
            &naccounts,
            |b, _| {
                b.iter(|| {
                    let root = db.merkle_root();
                    black_box(root)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_account_generation,
    benchmark_merkle_root_computation
);
criterion_main!(benches);

# Ledger

Rust implementation of the mina ledger

## Run tests:
```bash
cargo test --release
```

## Run tests on wasm:
```bash
export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--max-memory=4294967296"
wasm-pack test --release --chrome --headless -- -Z build-std=std,panic_abort # browser chrome
wasm-pack test --release --firefox --headless -- -Z build-std=std,panic_abort # browser firefox
```



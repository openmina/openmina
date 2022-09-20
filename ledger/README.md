# Ledger

Rust implementation of the mina ledger

## Run tests:
```bash
cargo test --release
```

## Run tests on wasm:
```bash
wasm-pack test --release --node -- --features in_nodejs # nodejs
wasm-pack test --release --chrome --headless # browser chrome
wasm-pack test --release --firefox --headless # browser firefox
```



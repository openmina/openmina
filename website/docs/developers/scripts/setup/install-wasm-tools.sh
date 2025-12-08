# Install wasm-pack for WebAssembly builds
cargo install wasm-pack

# Install wasm-bindgen CLI tool for generating WebAssembly bindings
cargo install --force wasm-bindgen-cli --version 0.2.106 --force

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

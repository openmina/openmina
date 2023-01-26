## How to build

### Prerequisites installation

#### Install system dependencies
```bash
apt install build-essential protobuf-compiler curl git
```

#### Set up Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup install nightly
```

#### Set up WASM
```bash
rustup target add wasm32-unknown-unknown
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
cargo +nightly install wasm-pack
```

### Clone OpenMina
```bash
git clone https://github.com/openmina/openmina
cd openmina
```

### Build WASM
```bash
rustup run nightly wasm-pack build --target web -d ../../pkg node/wasm
```

Output will be in `./pkg` directory.

To use it with [mina-frontend](https://github.com/openmina/mina-frontend),
copy contents of the `./pkg` directory inside `mina-frontend/src/assets/webnode/mina-rust/`

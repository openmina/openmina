# The Mina Web Node

The Mina Web Node is an in-browser non-consensus node capable of verifying the blockchain state, which is achieved by verifying the SNARK of the head block it receives from the network. 

This in-browser node is designed to be launched from any device capable of running an internet browser. Thanks to its very simple and fast set-up, it is suitable for everyone, even users who lack technical knowledge in this area, and it has very low hardware requirements, so it can be run even from low-CPU and low-memory devices such as smartphones.

## How to build

### Prerequisites for installation

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

The output will be in `./pkg` directory.

To use it with the [mina-frontend](https://github.com/openmina/mina-frontend),
copy the contents of the `./pkg` directory into `mina-frontend/src/assets/webnode/mina-rust/`

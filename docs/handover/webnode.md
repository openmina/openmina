# OpenMina Webnode Implementation

This document covers the WebAssembly (WASM) build target of OpenMina located in
`node/web/`.

## Overview

The webnode compiles the full OpenMina node to WebAssembly for browser
execution. It includes block production, transaction processing, SNARK
verification, and WebRTC-based P2P networking.

### Design Goals

- Run the full node stack in browsers without plugins
- Maintain compatibility with the main OpenMina implementation
- Support block production with browser-based proving
- Provide JavaScript API for web applications

## Architecture

### WASM Target

Builds as both `cdylib` and `rlib` crate types. Code is conditionally compiled
with `#[cfg(target_family = "wasm")]` guards.

#### Build Process

```bash
cd node/web
cargo +nightly build --release --target wasm32-unknown-unknown
wasm-bindgen --keep-debug --web --out-dir ../../frontend/src/assets/webnode/pkg ../../target/wasm32-unknown-unknown/release/openmina_node_web.wasm
```

Requires nightly toolchain and generates bindings for
`frontend/src/assets/webnode/pkg/`.

For complete setup instructions including circuit downloads and frontend
configuration, see [local-webnode.md](../local-webnode.md).

### Threading

Browser threading constraints require specific adaptations:

#### Rayon Setup

`init_rayon()` in `rayon.rs` configures the thread pool using
`num_cpus.max(2) - 1` threads. Must be called before SNARK verification.

#### Task Spawning

- `P2pTaskSpawner`: Uses `wasm_bindgen_futures::spawn_local()`
- `P2pTaskRemoteSpawner`: Routes tasks to main thread via
  `thread::start_task_in_main_thread()` because WebRTC APIs are main-thread only

## Features

Provides the same functionality as the native node:

- Transaction validation and application
- Ledger state management
- SNARK verification using browser-compiled circuits
- Consensus participation
- RPC interface for web applications

### Block Production

Supports block production with:

- Plain text or encrypted private keys (parsed in `parse_bp_key()`)
- Custom coinbase receivers
- Browser-based SNARK proving via `BlockProver::make()`

### Networking

#### WebRTC P2P Layer

- **Transport**: WebRTC DataChannels for browser-to-browser communication
- **Protocol**: Pull-based networking (see [P2P README](../p2p/readme.md))
- **Default Peer**:
  `/2bjYBqn45MmtismsAYP9rZ6Xns9snCcNsN1eDgQZB5s6AzY2CR2/https/webrtc3.webnode.openmina.com/443`
- **Channels**: 8 distinct DataChannels for different protocol types (see
  [P2P README](../p2p/readme.md#channels))

#### Network Configuration

```rust
initial_peers: Vec<P2pConnectionOutgoingInitOpts>
peer_discovery: !self.p2p_no_discovery
max_peers: Some(100)
```

## Implementation Details

### Key Files

#### `lib.rs` - Main Entry Point

- **`main()`**: Automatic WASM initialization
- **`run()`**: Primary node startup function
- **`build_env()`**: Build information export
- **`parse_bp_key()`**: Block producer key parsing

#### `node/builder.rs` - Node Construction

- **`NodeBuilder`**: Node configuration
- **Configuration Methods**: P2P setup, block production, verification
- **Default Peers**: Single hardcoded WebRTC peer for bootstrap

#### `node/mod.rs` - Type Definitions

- **Type Aliases**: `Node = openmina_node_common::Node<NodeService>`
- **Task Spawners**: P2P-specific spawning implementations for browser
  constraints

#### `rayon.rs` - Threading Setup

- **`init_rayon()`**: Required initialization for multi-threading
- **CPU Detection**: Automatic core count with minimum guarantees

### JavaScript Interface

#### Main Entry Point

```javascript
const rpcSender = await run(blockProducerKey, seedNodesUrl, genesisConfigUrl);
```

- `blockProducerKey`: Optional string or `[encrypted, password]` array
- `seedNodesUrl`: Optional URL returning newline-separated peer addresses
- `genesisConfigUrl`: Optional URL returning binary genesis config (defaults to
  `DEVNET_CONFIG`)

#### Setup

- `console_error_panic_hook` enables panic traces in browser console
- `keep_worker_alive_cursed_hack()` prevents worker termination (wasm-bindgen
  issue #2945)

### Performance

- Parallel SNARK verification using `num_cpus.max(2) - 1` threads
- Circuit reuse for verification operations
- 100 peer connection limit configured in `P2pLimits`
- Statistics collection via `gather_stats()` when enabled

## Dependencies

**WASM-specific**: `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`,
`console_error_panic_hook`, `gloo-utils`

**Core**: Standard OpenMina workspace crates plus `rayon` for threading

## Known Issues

- Worker lifecycle requires `keep_worker_alive_cursed_hack()` due to
  wasm-bindgen issue #2945
- WebRTC operations restricted to main thread
- Careful initialization ordering required

## Technical Debt

- TODO in `setup_node()` for seed nodes refactoring
- Single hardcoded default peer
- Commented HTTP client and peer loading code in `builder.rs`

## Usage

```javascript
// Basic startup
const rpc = await run();

// With block producer
const rpc = await run("private-key");
const rpc = await run([encryptedKey, "password"]);

// With custom configuration
const rpc = await run(key, peersUrl, genesisUrl);

// RPC access
const peers = await rpc.state().peers();
const stats = await rpc.stats().sync();
```

## Future Work

- Split prover to its own WASM heap
  (https://github.com/openmina/openmina/issues/1128)
- API for zkApp integration

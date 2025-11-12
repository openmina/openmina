---
sidebar_position: 2
title: Scenario Tests
description:
  Deterministic scenario-based testing using the mina-node-testing framework
slug: /developers/testing/scenario-tests
---

# Scenario Tests

## Overview

Scenario tests provide deterministic, scenario-based testing for complex
multi-node blockchain scenarios using the `mina-node-testing` framework. Tests
are structured as sequences of steps that can be recorded, saved, and replayed
deterministically across different environments.

## Architecture

### Core design principles

1. **Scenario-based testing**: Tests are structured as scenarios consisting of
   ordered sequences of steps that can be recorded, saved, and replayed
   deterministically
2. **State machine architecture**: Follows the Redux-style pattern used
   throughout the Mina Rust node for predictable state transitions
3. **Multi-implementation support**: Tests both Rust (the Mina Rust node) and
   OCaml (original Mina) nodes in the same scenarios
4. **Deterministic replay**: All tests can be replayed exactly using recorded
   scenarios with fixed random seeds and controlled time progression

### mina-node-testing framework

#### Test library (`node/testing/src/`)

The `mina-node-testing` library provides the core runtime infrastructure:

- **Cluster management**: Coordinates multiple node instances with precise
  lifecycle control
- **Scenario orchestration**: Manages scenario execution, step ordering, and
  synchronization
- **State monitoring**: Observes node state changes and validates transitions
- **Action dispatch**: Triggers actions across nodes with deterministic timing
- **Recording/replay**: Captures and reproduces test scenarios with complete
  fidelity

#### Test runner (`node/testing/src/bin/runner.rs`)

The test runner provides comprehensive scenario management:

- **Scenario execution**: Runs scenarios from templates or recorded files
- **Step-by-step control**: Allows precise control over scenario progression
- **Progress monitoring**: Tracks scenario execution progress with detailed
  logging
- **Error handling**: Captures and reports test failures with full context
- **Replay capability**: Reproduces exact scenarios for debugging

#### Scenario structure

Scenarios consist of two main components:

- **ScenarioInfo**: Metadata including scenario name, description, and
  configuration
- **ScenarioSteps**: Ordered sequence of actions to be executed

Common step types include:

- **Node management**: Adding, removing, and configuring nodes
- **Network operations**: Connecting nodes, establishing P2P connections
- **Time control**: Advancing time deterministically for reproducible tests
- **Event dispatch**: Triggering specific actions on nodes
- **Validation**: Checking conditions and timeouts
- **State assertions**: Verifying expected node states

## Testing capabilities

### Cluster management

The framework provides comprehensive cluster orchestration:

```rust
use mina_node_testing::scenario::{Scenario, ScenarioStep};

// Create cluster with deterministic configuration
let scenario = Scenario::builder()
    .with_info("multi_node_sync", "Test multi-node synchronization")
    .with_step(ScenarioStep::AddNode {
        node_id: "node1".to_string(),
        config: NodeConfig::rust_default()
    })
    .with_step(ScenarioStep::AddNode {
        node_id: "node2".to_string(),
        config: NodeConfig::rust_default()
    })
    .with_step(ScenarioStep::ConnectNodes {
        dialer: "node1".to_string(),
        listener: "node2".to_string()
    })
    .build();
```

### Deterministic time control

Precise time control ensures reproducible test outcomes:

```rust
// Advance time in controlled increments
scenario.add_step(ScenarioStep::AdvanceTime {
    duration: Duration::from_secs(30)
});

// Check conditions with deterministic timeouts
scenario.add_step(ScenarioStep::CheckTimeout {
    timeout: Duration::from_secs(60),
    condition: "nodes_synchronized".to_string()
});
```

### Cross-implementation testing

Seamless integration of Rust and OCaml nodes:

```rust
// Mix implementations in same scenario
scenario
    .with_step(ScenarioStep::AddNode {
        node_id: "rust_node".to_string(),
        config: NodeConfig::rust_default()
    })
    .with_step(ScenarioStep::AddNode {
        node_id: "ocaml_node".to_string(),
        config: NodeConfig::ocaml_default()
    })
    .with_step(ScenarioStep::TestInteroperability {
        nodes: vec!["rust_node".to_string(), "ocaml_node".to_string()]
    });
```

### Invariant checking system

Comprehensive invariant validation throughout scenario execution:

```rust
// Define invariants to check continuously
scenario.add_invariant(Invariant::new(
    "consistent_best_tip",
    |cluster| cluster.nodes_have_consistent_best_tip(),
    InvariantCheckFrequency::EveryStep
));

// Custom invariants for specific conditions
scenario.add_invariant(Invariant::new(
    "connection_stability",
    |cluster| cluster.all_connections_stable(),
    InvariantCheckFrequency::AfterTimeAdvancement
));
```

### Scenario inheritance

Build complex scenarios from simpler components:

```rust
// Base scenario for node setup
let base_scenario = Scenario::builder()
    .with_info("base_two_nodes", "Basic two-node setup")
    .with_step(ScenarioStep::AddNode { /* ... */ })
    .with_step(ScenarioStep::AddNode { /* ... */ })
    .build();

// Extended scenario inheriting base setup
let extended_scenario = base_scenario
    .extend()
    .with_info("two_nodes_with_blocks", "Two nodes producing blocks")
    .with_step(ScenarioStep::ProduceBlocks { count: 5 })
    .build();
```

## Scenario categories

### Bootstrap scenarios

Node initialization and basic functionality:

- **Single node bootstrap**: Basic node startup and initialization
- **Multi-node bootstrap**: Coordinated startup of multiple nodes
- **State recovery**: Node restart and state restoration
- **Configuration validation**: Testing different node configurations

### Network formation scenarios

Peer-to-peer network establishment:

- **Initial network formation**: Nodes discovering and connecting to peers
- **Peer discovery mechanisms**: Testing different discovery protocols
- **Network topology**: Various network connection patterns
- **Large-scale networks**: Testing with many nodes (10+ nodes)

### Synchronization scenarios

Blockchain state synchronization:

- **Block propagation**: Blocks spreading through the network
- **Fork resolution**: Handling competing blockchain forks
- **Catchup mechanisms**: Nodes synchronizing after downtime
- **State consistency**: Ensuring all nodes reach same state

### Transaction scenarios

Transaction processing and propagation:

- **Transaction pool**: Managing pending transactions
- **Transaction propagation**: Spreading transactions across network
- **Transaction validation**: Ensuring proper transaction processing
- **Mempool management**: Testing transaction pool behavior

### P2P networking scenarios

Low-level networking functionality:

- **Connection establishment**: Basic peer-to-peer connections
- **Message routing**: Proper message delivery and routing
- **Network partitions**: Handling network splits and merges
- **Connection recovery**: Reconnection after network failures
- **Protocol compatibility**: Testing different protocol versions

### Cross-implementation scenarios

Interoperability between implementations:

- **Rust-OCaml interoperability**: Mixed implementation networks
- **Protocol compliance**: Adherence to Mina protocol specifications
- **Message compatibility**: Cross-implementation communication
- **Consensus participation**: Shared consensus across implementations

## Advanced testing features

### Proof configuration and mocking

Control proof generation behavior for faster test execution:

```rust
use mina_node_testing::config::{ProofConfig, ClusterConfig};

// Disable proofs entirely for speed
let config = ClusterConfig::new()
    .with_proof_config(ProofConfig::Disabled);

// Use dummy/mock proofs
let config = ClusterConfig::new()
    .with_proof_config(ProofConfig::Dummy);

// Use real proofs (slower but comprehensive)
let config = ClusterConfig::new()
    .with_proof_config(ProofConfig::Real);
```

### Random seed control

Ensure deterministic test execution:

```rust
// Fixed seed for reproducible randomness
let scenario = Scenario::builder()
    .with_random_seed(12345)
    .with_info("deterministic_test", "Reproducible scenario");

// Different seed for variation testing
let scenario = Scenario::builder()
    .with_random_seed(67890)
    .with_info("variation_test", "Alternative execution path");
```

### Large-scale network testing

Test scenarios with many nodes:

```rust
// Create 10-node network efficiently
let scenario = Scenario::builder()
    .with_info("large_network", "10-node network formation")
    .with_steps((0..10).map(|i|
        ScenarioStep::AddNode {
            node_id: format!("node_{}", i),
            config: NodeConfig::rust_default()
        }
    ).collect())
    .with_step(ScenarioStep::ConnectAllNodes)
    .with_step(ScenarioStep::WaitForFullMesh);
```

### Debugging and introspection

Advanced debugging capabilities:

```rust
// Enable detailed logging
scenario.add_step(ScenarioStep::SetLogLevel {
    level: "trace".to_string(),
    components: vec!["p2p".to_string(), "consensus".to_string()]
});

// Take state snapshots
scenario.add_step(ScenarioStep::TakeSnapshot {
    name: "before_fork".to_string()
});

// Compare states between nodes
scenario.add_step(ScenarioStep::CompareNodeStates {
    nodes: vec!["node1".to_string(), "node2".to_string()],
    fields: vec!["best_tip".to_string(), "peer_count".to_string()]
});
```

## Running tests

The list of available tests can be found by running:

```
cargo run --release --features scenario-generators,p2p-webrtc \
  --bin mina-node-testing -- scenarios-list
```

## CI vs Local Test Execution

There are important differences between how tests run in CI versus locally:

### CI Environment (with Sidecar Container)

In CI, tests run with a
**[network debugger sidecar container](https://github.com/openmina/mina-network-debugger)**
that provides deep network inspection capabilities:

- **Network monitoring**: A `bpf-recorder` sidecar container runs alongside test
  nodes
- **Binary execution**: CI builds and uses specific scenario binaries based on
  directory structure:
  - Built using `make build-tests` and `make build-tests-webrtc`
  - `solo_node/` directory scenarios → dedicated solo node test binaries
  - `multi_node/` directory scenarios → multi-node test binaries
  - `p2p/` directory scenarios → P2P networking test binaries
- **Debugger integration**: Automatic network traffic capture and analysis
- **Port configuration**: Debugger runs on port 8000 (configured for CI
  environment)
- **Enhanced observability**: Full message tracing, connection monitoring, and
  protocol analysis

The sidecar container provides:

- Real-time network connection tracking
- Message-level protocol inspection
- Connection lifecycle monitoring
- Stream-level debugging capabilities
- HTTP API for accessing captured network data

### Local Environment (Scenario-run)

Locally, tests run through the scenario-run interface with simplified execution:

- **Scenario names**: Tests are referenced by name (e.g., `p2p-signaling`,
  `solo-node-bootstrap`)
- **Direct execution**: Single binary (`mina-node-testing`) handles all scenario
  types
- **Optional debugger**: Network debugger can be enabled with `--use-debugger`
  flag
- **Simplified setup**: No external container dependencies
- **Development focus**: Optimized for rapid iteration and debugging

### Key Differences

| Aspect            | CI (Sidecar)             | Local (Scenario-run)        |
| ----------------- | ------------------------ | --------------------------- |
| **Execution**     | Directory-based binaries | Name-based scenarios        |
| **Network Debug** | Always enabled (sidecar) | Optional (`--use-debugger`) |
| **Observability** | Full network inspection  | Basic logging               |
| **Setup**         | Container orchestration  | Single binary               |
| **Use Case**      | Comprehensive testing    | Development iteration       |

### Running with Network Debugger Locally

To enable similar debugging capabilities locally:

```bash
# Enable network debugger for detailed inspection
cargo run --release --bin mina-node-testing -- \
  scenarios-generate --use-debugger --name scenario-name
```

The local debugger spawns a `bpf-recorder` process that provides similar network
monitoring capabilities as the CI sidecar, though without the container
isolation.

## Troubleshooting

### Workflow Requirements

- **scenarios-run**: Expects pre-existing scenario files in
  `node/testing/res/scenarios/`
- **scenarios-generate**:
  - Default (`--output=stdout`): Runs scenarios and outputs to stdout, no JSON
    files created
  - With `--output=json`: Runs scenarios and saves them as JSON files in
    `node/testing/res/scenarios/`

#### Understanding Scenario Load/Save Implementation

For detailed technical information about how scenarios are loaded and saved, see
the
[scenario module source code](https://github.com/o1-labs/mina-rust/blob/develop/node/testing/src/scenario/mod.rs).

### Scenario generation and replay

The `mina-node-testing` framework supports both scenario generation and replay:

```bash
# Generate and run scenarios (default: output to stdout, no JSON file saved)
cargo run --release --features scenario-generators --bin mina-node-testing -- \
  scenarios-generate --name record-replay-block-production

# Generate and save scenarios to JSON files
cargo run --release --features scenario-generators --bin mina-node-testing -- \
  scenarios-generate --name record-replay-block-production --output=json

# Generate WebRTC scenarios (requires additional p2p-webrtc feature)
cargo run --release --features scenario-generators,p2p-webrtc \
  --bin mina-node-testing -- scenarios-generate --name p2p-signaling --output=json

# Generate all scenarios and save to JSON
cargo run --release --features scenario-generators --bin mina-node-testing -- \
  scenarios-generate --output=json

# Replay existing scenarios (requires JSON files from scenarios-generate --output=json)
cargo run --release --bin mina-node-testing -- scenarios-run --name p2p-signaling
```

### Key execution features

- **Feature-based compilation**: Use `--features` to enable specific test
  capabilities
- **Scenario generation**:
  - Run scenarios directly with `--output=stdout` (default)
  - Generate JSON files with `--output=json` for later replay
- **Scenario replay**: Execute previously generated JSON scenarios using
  `scenarios-run`
- **Named scenarios**: Reference scenarios by name for consistent execution

### Environment configuration

For network connectivity in testing environments, you may need to configure:

```bash
# Enable connection to replayer service (used in CI)
export REPLAYER_MULTIADDR="/dns4/mina-rust-ci-1-libp2p.gcp.o1test.net/tcp/8302/p2p/12D3KooWNazk9D7RnbHFaPEfrL7BReAKr3rDRf7PivS2Lwx3ShAA"

# Allow local address discovery
export MINA_DISCOVERY_FILTER_ADDR=false

# Maintain connections with unknown streams (for replayer compatibility)
export KEEP_CONNECTION_WITH_UNKNOWN_STREAM=true
```

The `REPLAYER_MULTIADDR` variable specifies the multiaddress for connecting to
the Mina protocol network's primary TCP proxy, enabling scenario tests to
interact with live network components when needed.

## Best practices

### Scenario design principles

1. **Start simple**: Begin with single-node scenarios before advancing to
   multi-node complexity
2. **Use scenario inheritance**: Build complex scenarios from proven simpler
   components
3. **Minimize time advancement**: Use smallest time increments necessary for
   deterministic behavior
4. **Add comprehensive invariants**: Define and check system properties
   throughout test execution
5. **Use fixed random seeds**: Ensure reproducible test outcomes across
   environments
6. **Handle resource cleanup**: Properly clean up nodes and network resources
   after tests

### Deterministic testing guidelines

1. **Control all randomness**: Use fixed seeds for any random operations
2. **Minimize external dependencies**: Avoid relying on external services or
   timing
3. **Use precise time control**: Advance time in controlled, minimal increments
4. **Validate state transitions**: Check node states after each significant step
5. **Test failure scenarios**: Include network partitions, node failures, and
   edge cases

### Debugging and troubleshooting

1. **Enable comprehensive logging**: Use detailed logging levels for failing
   scenarios
2. **Use state snapshots**: Capture node state at critical points for comparison
3. **Replay with variations**: Test with different seeds and timing to isolate
   issues
4. **Compare node states**: Verify consistency between nodes at checkpoint steps
5. **Validate invariants**: Ensure system properties hold throughout scenario
   execution

### Performance optimization

1. **Use proof mocking**: Disable or mock proofs during development for faster
   iteration
2. **Minimize node count**: Use smallest number of nodes that demonstrate the
   behavior
3. **Batch similar operations**: Group related steps together for efficiency
4. **Profile resource usage**: Monitor memory and CPU usage during large
   scenarios

## Related Documentation

- [Testing Framework Overview](testing-framework): Main testing documentation
- [Unit Tests](unit-tests): Basic component testing
- [P2P Tests](p2p-tests): P2P networking specific tests
- [OCaml Node Tests](ocaml-node-tests): OCaml interoperability testing

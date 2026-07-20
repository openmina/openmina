# OpenMina Testing Infrastructure Handover Document

## Overview

The OpenMina testing infrastructure provides scenario-based testing for
multi-node blockchain scenarios. Tests are structured as sequences of steps that
can be recorded, saved, and replayed deterministically.

## Architecture

### Core Design Principles

1. **Scenario-Based Testing**: Tests are structured as scenarios - sequences of
   steps that can be recorded, saved, and replayed deterministically
2. **State Machine Architecture**: Follows the Redux-style pattern used
   throughout OpenMina
3. **Multi-Implementation Support**: Tests both Rust (OpenMina) and OCaml
   (original Mina) nodes
4. **Deterministic Replay**: All tests can be replayed exactly using recorded
   scenarios

### Key Components

#### 1. Test Library (`node/testing/src/lib.rs`)

- Provides test runtime setup and synchronization
- Manages global test gates to ensure sequential execution
- Initializes tracing and thread pools

#### 2. Test Runner (`node/testing/src/main.rs`)

The testing binary provides three commands, though the `server` command
currently has a clap configuration bug:

```bash
# Generate new test scenarios (requires scenario-generators feature)
cargo run --bin openmina-node-testing --features=scenario-generators -- scenarios-generate

# Replay recorded scenarios
cargo run --bin openmina-node-testing -- scenarios-run --name "ScenarioName"

# Server command exists but has a clap argument conflict bug
```

Note: Most testing is done via standard `cargo test` commands rather than the
binary.

#### 3. Scenario Framework (`node/testing/src/scenarios/mod.rs`)

Contains extensive predefined test scenarios organized into categories:

- **Solo Node Tests**: Single node sync and bootstrap tests
- **Multi-Node Tests**: Network connectivity and consensus tests
- **P2P Tests**: Connection handling and peer discovery
- **Simulation Tests**: Long-running network simulations
- **Record/Replay Tests**: Replaying recorded scenarios

## Scenario System

### Scenario Structure

Each scenario consists of:

- **ScenarioInfo**: Metadata (ID, description, parent scenario)
- **ScenarioSteps**: Ordered list of actions

### Common Step Types

```rust
- AddNode { config }                    // Add a new node to the cluster
- ConnectNodes { dialer, listener }     // Connect two nodes
- AdvanceTime { by_nanos }              // Advance monotonic time
- Event { node_id, event }              // Dispatch specific events
- CheckTimeouts { node_id }             // Process timeout events
- AdvanceNodeTime { node_id, by_nanos } // Advance time for specific node
```

### Scenario Inheritance

Scenarios can have parent scenarios, allowing test composition:

```
ParentScenario (sets up base state)
    └── ChildScenario (builds on parent state)
```

## Testing Infrastructure

### NodeTestingService (`node/testing/src/service/mod.rs`)

Wraps the real `NodeService` with testing capabilities:

- **Event Management**: Tracks events with IDs for replay
- **Time Control**: Allows precise time advancement
- **Proof Mocking**: Supports dummy proofs for faster tests (ProofKind::Dummy)
- **Service Mocking**: Mock block production, SNARK verification, etc.

### Cluster Management (`node/testing/src/cluster/mod.rs`)

Manages multiple nodes in a test environment:

```rust
let mut cluster = Cluster::new(cluster_config);
let mut runner = ClusterRunner::new(&mut cluster, |_step| {});
let rust_node_id = runner.add_rust_node(config);
let ocaml_node_id = runner.add_ocaml_node(config);
```

Features:

- Automatic port allocation
- Account key management
- Network topology control
- Debugger integration

### OCaml Node Limitations

When including OCaml nodes in test scenarios, there are several important
limitations compared to Rust nodes:

**Time Control:**

- OCaml nodes use real wall-clock time and cannot be controlled via
  `AdvanceTime` or `AdvanceNodeTime` steps
- Only Rust nodes support deterministic time advancement
- This can cause timing-dependent test failures when OCaml and Rust nodes get
  out of sync

**Visibility and Debugging:**

- OCaml nodes are "black boxes" - we cannot inspect their internal state like we
  can with Rust nodes
- No access to OCaml node's internal execution, state changes, or data
  structures
- Limited logging and debugging capabilities compared to Rust nodes
- Cannot use invariant checking on OCaml node state

**Network Control:**

- Cannot manually disconnect OCaml peers using test framework commands
- Network topology changes must be done externally or through OCaml node's own
  mechanisms
- Limited control over OCaml node's P2P behavior and connection management

**Behavioral Control:**

- No control over OCaml node's internal execution flow or decision-making
- Cannot trigger specific OCaml node behaviors on demand
- Cannot guarantee that expected operations will be executed at all
- This limits the determinism of tests involving OCaml nodes

**Testing Implications:**

- Tests with OCaml nodes are inherently less deterministic
- Focus should be on testing interoperability rather than detailed protocol
  behavior
- Use OCaml nodes primarily for cross-implementation validation
- Consider using Rust-only scenarios when precise control is needed

### Potential OCaml Node Testing Improvements

To improve OCaml node testing capabilities, the following changes could be made
to the OCaml implementation:

**Deterministic Time Control:**

- Add support for controllable time advancement instead of wall-clock time
- Implement time mocking or virtual time system that can be controlled by the
  test framework
- This would enable synchronization between OCaml and Rust nodes in tests

**Testing API:**

- Expose internal state inspection endpoints for testing purposes
- Add hooks or callbacks for test frameworks to monitor internal execution
- Implement test-specific logging and debugging interfaces

**Network Control:**

- Add testing APIs to manually control P2P connections
- Implement test hooks for network topology manipulation
- Provide mechanisms to trigger specific network behaviors on demand

**Behavioral Control:**

- Add test-specific triggers for protocol operations
- Implement deterministic execution modes for testing
- Provide APIs to control or observe internal decision-making processes

**Implementation Notes:**

- These improvements would require coordination with the OCaml Mina team
- Changes should be designed to not affect production behavior
- Testing improvements could be implemented as optional testing-only features

## Invariant Checking System

### What are Invariants?

Invariants are properties that must always hold true during execution. The
testing framework continuously checks these invariants to catch bugs early.

### Invariant Interface (`node/invariants/src/lib.rs`)

```rust
pub trait Invariant {
    type InternalState: 'static + Send + Default;

    fn is_global(&self) -> bool { false }
    fn triggers(&self) -> &[ActionKind];

    fn check<S: Service>(
        self,
        internal_state: &mut Self::InternalState,
        store: &Store<S>,
        action: &ActionWithMeta,
    ) -> InvariantResult;
}
```

### Built-in Invariants

1. **NoRecursion**: Prevents recursive action dispatching
2. **P2pStatesAreConsistent**: Ensures P2P state consistency across nodes
3. **TransitionFrontierOnlySyncsToBetterBlocks**: Validates blockchain
   synchronization logic

### Creating Custom Invariants

```rust
impl Invariant for MyInvariant {
    type InternalState = ();  // Or custom state type

    fn triggers(&self) -> &[ActionKind] {
        // Return actions that should trigger this check
        &[ActionKind::SomeAction]
    }

    fn check<S: Service>(
        self,
        _internal_state: &mut Self::InternalState,
        store: &Store<S>,
        action: &ActionWithMeta,
    ) -> InvariantResult {
        // Check your invariant condition using store.state()
        if condition_violated {
            InvariantResult::Violated("Description of violation")
        } else {
            InvariantResult::Ok
        }
    }
}
```

## Test Patterns and Examples

### 1. Single Node Bootstrap Test

**File**:
[`solo_node/bootstrap.rs`](../../../node/testing/src/scenarios/solo_node/bootstrap.rs)

**Testing Pattern**: Validates that a single Rust node can bootstrap against
real blockchain data from a replayer.

**Key Techniques**:

- **Replayer Integration**: Uses a host replayer with actual blockchain data
  rather than synthetic test data
- **Multi-phase Validation**: Separately checks staking ledger sync, next epoch
  ledger sync, and transition frontier sync
- **Time Coordination**: Carefully manages timestamp alignment with recorded
  blockchain data to avoid validation failures

### 2. Multi-Node Network Test

**File**:
[`multi_node/sync_4_block_producers.rs`](../../../node/testing/src/scenarios/multi_node/sync_4_block_producers.rs)

**Testing Pattern**: Tests consensus participation and synchronization across
multiple block-producing nodes.

**Key Techniques**:

- **Block Producer Configuration**: Creates nodes with actual block producer
  keys and configs
- **Topology Control**: Explicitly connects nodes in controlled patterns rather
  than full mesh
- **Consensus Validation**: Verifies that all nodes reach the same blockchain
  state through consensus participation

### 3. Cross-Implementation Test

**File**:
[`multi_node/connection_discovery.rs`](../../../node/testing/src/scenarios/multi_node/connection_discovery.rs)

**Testing Pattern**: Validates interoperability between Rust (OpenMina) and
OCaml (original Mina) implementations.

**Key Techniques**:

- **Implementation Bridging**: Tests communication between different protocol
  implementations
- **Peer Discovery**: Validates Kademlia-based peer discovery across
  implementation boundaries
- **Bidirectional Validation**: Ensures both implementations can discover and
  communicate with each other

## Advanced Testing Examples

### P2P Connection Race Condition Testing

**File**:
[`p2p/basic_connection_handling.rs`](../../../node/testing/src/scenarios/p2p/basic_connection_handling.rs) -
`SimultaneousConnections`

**Testing Pattern**: Race conditions in P2P connection establishment where both
nodes initiate connections simultaneously.

**Key Techniques**:

- Tests **race conditions** in distributed systems
- Uses **proper async testing** with timeout handling
- Validates **connection deduplication** - system handles simultaneous
  connections gracefully
- Employs **steady state verification** - waits for system to settle before
  assertions

### Deterministic Replay Testing

**File**:
[`record_replay/block_production.rs`](../../../node/testing/src/scenarios/record_replay/block_production.rs) -
`RecordReplayBlockProduction`

**Testing Pattern**: Deterministic execution validation for blockchain consensus
logic.

**Key Techniques**:

- **Determinism validation** - critical for blockchain consensus
- **Record-replay pattern** - captures and reproduces exact execution sequences
- **State comparison** - verifies identical outcomes across runs
- **Non-determinism detection** - catches sources of randomness that could break
  consensus

### VRF Epoch Boundary Testing

**File**:
[`multi_node/vrf_epoch_bounds_evaluation.rs`](../../../node/testing/src/scenarios/multi_node/vrf_epoch_bounds_evaluation.rs) -
`MultiNodeVrfEpochBoundsEvaluation`

**Testing Pattern**: VRF (Verifiable Random Function) evaluation across epoch
boundaries in blockchain consensus.

**Key Techniques**:

- **Time-dependent testing** - validates VRF evaluation across epoch boundaries
- **Block production control** - uses `produce_blocks_until` with custom
  predicates
- **State inspection** - accesses actual `vrf_evaluator().latest_evaluated_slot`
  state
- **Epoch transition validation** - tests critical blockchain timing at slot
  boundaries

### Large-Scale Network Testing

**File**:
[`multi_node/pubsub_advanced.rs`](../../../node/testing/src/scenarios/multi_node/pubsub_advanced.rs) -
`MultiNodePubsubPropagateBlock`

**Testing Pattern**: Block propagation through gossip networks at scale.

**Key Techniques**:

- **Scale testing** - validates behavior with 10+ nodes using Simulator
- **Action monitoring** - tracks P2P message propagation in real-time
- **Graph visualization** - generates DOT format network graphs for debugging
- **Deterministic recording** - captures all state transitions for replay
- **Blockchain simulation** - tests actual block production and gossip
  propagation

## Running Tests

### Scenario Generation and Replay

```bash
# Generate specific scenarios (requires scenario-generators feature)
cargo run --release --features scenario-generators --bin openmina-node-testing -- scenarios-generate --name record_replay_block_production

# Generate WebRTC scenarios (requires additional p2p-webrtc feature)
cargo run --release --features scenario-generators,p2p-webrtc --bin openmina-node-testing -- scenarios-generate --name webrtc_p2p_signaling

# Generate multi-node scenarios
cargo run --release --features scenario-generators --bin openmina-node-testing -- scenarios-generate --name multi-node-pubsub-propagate-block

# Replay existing scenarios
cargo run --release --bin openmina-node-testing -- scenarios-run --name "ScenarioName"
```

## Best Practices

### 1. Writing New Tests

- Start with existing scenario as template
- Use parent scenarios for common setup
- Record all non-deterministic inputs
- Add invariants for critical properties

### 2. Debugging Failed Tests

- Use `--nocapture` to see all logs
- Enable network debugger for visualization
- Replay scenarios with added logging
- Check invariant violations first

### 3. Performance Considerations

- Use `ProofKind::Dummy` for logic tests
- Minimize time advancement steps
- Batch similar operations
- Clean up resources properly

## Common Issues and Solutions

### 1. Non-Deterministic Failures

- **Issue**: Test passes sometimes but fails others
- **Solution**: Ensure all randomness uses fixed seeds, check for timing
  dependencies

### 2. Port Conflicts

- **Issue**: "Address already in use" errors
- **Solution**: Use unique port ranges per test, ensure proper cleanup

### 3. Slow Test Execution

- **Issue**: Tests take too long
- **Solution**: Use dummy proofs, reduce node count, optimize wait conditions

### 4. Invariant Violations

- **Issue**: Invariant check fails during test
- **Solution**: Check logs for violation details, add debugging, fix state
  inconsistency

## Advanced Features

### Proof Configuration

Control proof generation for faster testing:

```rust
// From cluster configuration - use dummy proofs to speed up tests
let mut cluster_config = ClusterConfig::new(None)?;
cluster_config.set_proof_kind(ProofKind::Dummy);        // Fastest - no proof verification
// cluster_config.set_proof_kind(ProofKind::ConstraintsChecked);  // Medium - check constraints only
// cluster_config.set_proof_kind(ProofKind::Full);       // Slowest - full proof generation/verification
```

### Custom Action Monitoring

Track specific network events during tests:

```rust
// From pubsub_advanced.rs - monitor gossip message propagation
let factory = || {
    move |_id, state: &node::State, _service: &NodeTestingService, action: &ActionWithMeta| {
        match action.action() {
            Action::P2p(P2pAction::Network(P2pNetworkAction::Pubsub(
                P2pNetworkPubsubAction::OutgoingMessage { peer_id },
            ))) => {
                // Track block propagation for visualization
                let pubsub_state = &state.p2p.ready().unwrap().network.scheduler.broadcast_state;
                // Process gossip messages...
                false
            }
            _ => false,
        }
    }
};
```

## Notes

- Tests are scenarios that can be recorded and replayed
- Invariants continuously validate system properties
- Multi-node and cross-implementation testing is well-supported
- Debugging tools help diagnose complex issues

For additional examples and patterns, refer to:

- Test scenarios: `node/testing/src/scenarios/`
- Testing documentation: [`docs/testing/`](../../testing/) - Contains detailed
  test descriptions, troubleshooting guides, and testing methodology
  documentation

## Appendix: Block Replayer Tool

**Current Status**: An unfinished prototype exists in the
`tools/block-replayer/` directory on the `feat/basic-block-replayer` branch but
was never integrated into the testing workflow.

**Purpose**: Sequential block replay from genesis to validate that the node's
transaction logic, ledger operations, and block processing are correct against
real blockchain data. Could also be used to test proof verification (blocks,
completed works) and on devnet, for blocks produced by accounts of which the
private key is available, to reproduce the proof and ensure the prover works
correctly by ensuring it should have been able to produce all those proofs by
itself.

**Value Proposition**:

- **Transaction Logic Validation**: Ensures transaction processing matches
  expected behavior from historical blocks
- **Ledger Operation Testing**: Validates ledger state transitions and account
  updates
- **Block Processing Verification**: Tests the complete block application
  pipeline
- **Real-World Coverage**: Uses actual blockchain data rather than synthetic
  test scenarios
- **Regression Testing**: Catches regressions in core blockchain logic

**Recommendation**: The new team should complete or rewrite this tool as a
standalone testing utility. This would provide validation of core blockchain
logic that is complementary to but separate from the scenario-based testing
framework.

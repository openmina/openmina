# OpenMina Architecture & Code Walk-through

## Table of Contents

1. [Introduction](#introduction)
2. [Architecture Philosophy](#architecture-philosophy)
3. [State Machine Architecture](#state-machine-architecture)
4. [Core Components Overview](#core-components-overview)
5. [Network Configuration System](#network-configuration-system)
6. [Code Organization Patterns](#code-organization-patterns)
7. [Testing & Debugging](#testing--debugging)
8. [Development Guidelines](#development-guidelines)
9. [Communication Patterns](#communication-patterns)

## Introduction

OpenMina uses a Redux-inspired architecture pattern where application state is
centralized and all state changes flow through a predictable action dispatch
system. The system is designed as one large state machine composed of smaller,
domain-specific state machines (P2P networking, block production, consensus,
etc.) that work together.

All CPU-intensive operations, I/O, and non-deterministic operations are moved to
services - separate components that interact with the outside world and run in
their own threads. This separation ensures the core state machine remains
deterministic, making the system predictable, testable, and debuggable.

> **Next Steps**: After this overview, read
> [State Machine Structure](state-machine-structure.md) for implementation
> details, then [Project Organization](organization.md) for codebase navigation.

### Key Design Principles

- **Deterministic execution** - Given same inputs, behavior is always identical
- **Pure state management** - State changes only through reducers
- **Effect isolation** - Side effects separated from business logic
- **Component decoupling** - Clear boundaries between subsystems

## Architecture Philosophy

The architecture distinguishes between two fundamental types of components:

### State Machine Components (Stateful Actions)

- Manage core application state through pure functions
- Business logic resides in reducers with controlled state access
- Designed for determinism and predictability
- Interact with services only via effectful actions

### Service Components (Effectful Actions)

- Handle "outside world" interactions (network, disk, heavy computation)
- Run asynchronously to keep state machine responsive
- Minimal internal state - decision-making stays in state machine
- Communicate back via Events wrapped in actions

This separation ensures the core state management remains deterministic and
testable while side effects are handled in a controlled manner.

## State Machine Architecture

### Core Concepts

#### State

State is the central concept in the architecture - it represents the entire
application's data at any point in time. The global state is composed of smaller
domain-specific states:

```rust
pub struct State {
    pub p2p: P2pState,
    pub transition_frontier: TransitionFrontierState,
    pub snark_pool: SnarkPoolState,
    pub transaction_pool: TransactionPoolState,
    pub block_producer: BlockProducerState,
    // ... etc
}
```

Each component manages its own state structure, often using enums to represent
different stages of operations:

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32, started_at: Timestamp },
    Connected { peer_info: PeerInfo },
    Error { reason: String },
}
```

State is directly mutable by reducers as an optimization - rather than returning
new state, reducers modify the existing state in place.

#### Actions

Actions represent state transitions in the system. They are nested
hierarchically by context:

```rust
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    P2p(P2pAction),
    Ledger(LedgerAction),
    TransitionFrontier(TransitionFrontierAction),
    // ... etc
}
```

Actions are divided into two categories:

- **Stateful Actions**: Update state and dispatch other actions (handled by
  reducers)
- **Effectful Actions**: Thin wrappers for service interactions (handled by
  effects)

#### Enabling Conditions

Every action must implement `EnablingCondition` to prevent invalid state
transitions:

```rust
pub trait EnablingCondition<State> {
    fn is_enabled(&self, state: &State, time: Timestamp) -> bool;
}
```

Reducers, after performing a state update, will attempt to advance the state
machine in all directions that make sense from that point by dispatching
multiple potential next actions. However, it is the enabling conditions that
ultimately decide which of these transitions actually proceed. This creates a
natural flow where reducers propose all possible next steps, and enabling
conditions act as gates that filter out invalid paths based on the current
state.

For example, a reducer might dispatch actions to send messages to all connected
peers, but enabling conditions will filter out actions for peers that have since
disconnected.

#### Reducers (New Style)

In the new architecture, reducers handle both state updates and action
dispatching:

```rust
impl ComponentState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: ComponentActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else { return };

        match action {
            ComponentAction::SomeAction { data } => {
                // Phase 1: State updates
                state.field = data.clone();

                // Phase 2: Dispatch follow-up actions
                let dispatcher = state_context.into_dispatcher();
                // Or use into_dispatcher_and_state() for global state access
                // let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                dispatcher.push(ComponentAction::NextAction { ... });
            }
        }
    }
}
```

The `Substate` context enforces separation between state mutation and action
dispatching phases.

#### Effects (New Style)

Effects are now thin wrappers that call service methods:

```rust
impl EffectfulAction {
    pub fn effects<S: Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        match self {
            EffectfulAction::LoadData { id } => {
                store.service.load_data(id.clone());
            }
            EffectfulAction::ComputeProof { input } => {
                store.service.compute_proof(input.clone());
            }
        }
    }
}
```

Effects do NOT dispatch actions - they only interact with services. Services
communicate results back via Events.

### Execution Model

#### Single-Threaded Concurrent State Machines

A critical architectural principle: **all state machines run in a single thread
but operate concurrently**.

**Concurrent, Not Parallel:**

- Multiple state machines can be in different phases of their lifecycles
  simultaneously
- A connection may be `Pending` while VRF evaluation is `InProgress` and block
  production is `Idle`
- Only one action processes at a time - no race conditions or synchronization
  needed

**Single-Threaded Benefits:**

- **Deterministic execution** - Actions process in a predictable order
- **Simplified debugging** - No thread synchronization issues
- **State consistency** - No locks or atomic operations needed
- **Replay capability** - Exact reproduction of execution sequences

**Example Flow:**

```
Time 1: P2pConnectionAction::Initialize → P2P state becomes Connecting
Time 2: VrfEvaluatorAction::BeginEpoch → VRF state becomes Evaluating
Time 3: P2pConnectionAction::Success → P2P state becomes Ready
Time 4: VrfEvaluatorAction::Continue → VRF continues evaluation
```

Each action executes atomically, but multiple state machines progress
independently.

**Services and Threading:** While the state machine is single-threaded,
CPU-intensive work runs in dedicated service threads:

- Main thread: Redux store, all state transitions
- Service threads: Proof generation, cryptographic operations, I/O
- Communication: Services send events back via channels

This design keeps the state machine responsive while isolating non-deterministic
operations.

### Defensive Programming with `bug_condition!`

The codebase uses a `bug_condition!` macro for defensive programming and
invariant checking:

```rust
P2pChannelsRpcAction::RequestSend { .. } => {
    let Self::Ready { local, .. } = rpc_state else {
        bug_condition!(
            "Invalid state for `P2pChannelsRpcAction::RequestSend`, state: {:?}",
            rpc_state
        );
        return Ok(());
    };
    // Continue processing...
}
```

**Purpose**: `bug_condition!` marks code paths that should be unreachable if
enabling conditions work correctly. It provides a safety net for catching
programming logic errors.

**Behavior**:

- **Development** (`OPENMINA_PANIC_ON_BUG=true`): Panics immediately to catch
  bugs early
- **Production** (default): Logs error and continues execution gracefully

**Relationship to Enabling Conditions**:

1. Enabling conditions prevent invalid actions from reaching reducers
2. `bug_condition!` double-checks the same invariants in reducers
3. If `bug_condition!` triggers, it indicates a mismatch between enabling
   condition logic and reducer assumptions

This is **not error handling** - it's invariant checking for scenarios that
should never occur in correct code.

### State Machine Inputs

The state machine has three types of inputs ensuring deterministic behavior:

1. **Events** - External data from services wrapped in
   `EventSourceNewEventAction`
2. **Time** - Attached to every action via `ActionMeta`
3. **Synchronous service returns** - Avoided when possible

This determinism enables recording and replay for debugging.

## Core Components Overview

### Node (`node/`)

The main orchestrator containing:

**State Machine Components:**

- Core state/reducer/action management
- Block producer scheduling
- Transaction/SNARK pools
- Transition frontier (blockchain state) - _Note: Still uses old-style
  architecture_
- RPC request handling
- Fork resolution logic (with core consensus rules in `core/src/consensus.rs`)

**Service Components:**

- Block production service (prover interactions)
- Ledger service (database operations)
- External SNARK worker coordination
- Event source (aggregates external events)

### P2P Networking (`p2p/`)

Manages peer connections and communication through two distinct network layers:

**State Machine Components:**

- Connection lifecycle management
- Channel state machines that abstract over the differences between the two
  networks
- Channel management (RPC, streaming)
- Peer discovery (Kademlia DHT)
- Message routing (Gossipsub)

**Dual Network Architecture:**

_libp2p-based Network:_

- Used by native node implementations
- Transport protocols (TCP, custom WebRTC)
- Security (Noise handshake)
- Multiplexing (Yamux)
- Protocol negotiation

_WebRTC-based Network:_

- Used by webnode (browser-based node)
- Direct WebRTC transport implementation
- Different design pattern from libp2p
- Optimized for browser constraints

### SNARK Verification (`snark/`)

Handles zero-knowledge proof verification:

**State Machine Components:**

- Block verification state
- SNARK work verification
- Transaction proof verification

**Service Components:**

- Async proof verification services
- Batching for efficiency

### Ledger (`ledger/`)

A comprehensive Rust port of the OCaml Mina ledger with identical business
logic:

**Core Components:**

- **BaseLedger trait** - Fundamental ledger interface for account management and
  Merkle operations
- **Mask system** - Layered ledger views with copy-on-write semantics for
  efficient state management
- **Database** - In-memory account storage and Merkle tree management

**Transaction Processing:**

- **Transaction Pool** - Fee-based ordering, sender queue management, nonce
  tracking
- **Staged Ledger** - Transaction application and block validation
- **Scan State** - Parallel scan tree for SNARK work coordination

**Advanced Features:**

- **Proof System Integration** - Transaction, block, and zkApp proof
  verification using Kimchi
- **zkApp Support** - Full zkApp transaction processing with account updates and
  permissions
- **Sparse Ledger** - Efficient partial ledger representation for SNARK proof
  generation

**OCaml Compatibility:**

- Direct port maintaining same Merkle tree structure, transaction validation
  rules, and account model
- Memory-only implementation adapted to Rust idioms (Result types, ownership
  model)

_For detailed documentation, see [`ledger-crate.md`](ledger-crate.md)_

### Supporting Components

- **Core Types** (`core/`) - Shared data structures
- **Cryptography** (`vrf/`, `poseidon/`) - Crypto primitives
- **Serialization** (`mina-p2p-messages/`) - Network messages

## Network Configuration System

OpenMina supports multiple networks through a centralized configuration system
defined in `core/src/network.rs`:

### Network Types

- **Devnet** (`NetworkId::TESTNET`) - Development and testing network
- **Mainnet** (`NetworkId::MAINNET`) - Production Mina network

### Configuration Components

Each network configuration includes:

- **Cryptographic Parameters**: Network-specific signature prefixes and hash
  parameters
- **Circuit Configuration**: Directory names and circuit blob identifiers for
  each proof type
- **Default Peers**: Bootstrap peers for initial P2P connection
- **Constraint Constants**: Consensus parameters like ledger depth, work delay,
  block timing
- **Fork Configuration**: Hard fork parameters including state hash and
  blockchain length

### Configuration Initialization

1. **Global Access**: `NetworkConfig::global()` provides access to the active
   configuration
2. **Network Selection**: `NetworkConfig::init(network_name)` sets the global
   config once
3. **Service Integration**: All services access network parameters through the
   global config

This design ensures OpenMina can operate on different Mina networks while
maintaining protocol compatibility.

## Code Organization Patterns

### New Architecture Style

Most state machine components follow the new pattern with:

1. **Substate Access** - Fine-grained state control
2. **Unified Reducers** - Handle both state updates and action dispatching in
   two enforced phases
3. **Thin Effects** - Only wrap service calls
4. **Callbacks** - Enable decoupled component communication
5. **Clear Separation** - Stateful vs Effectful actions

Example structure:

```rust
// Stateful action with reducer
impl WatchedAccountsState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: WatchedAccountsActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else { return };

        match action {
            WatchedAccountsAction::Add { pub_key } => {
                // Update state
                state.insert(pub_key.clone(), WatchedAccountState { ... });

                // Dispatch follow-up
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(WatchedAccountsAction::LedgerInitialStateGetInit {
                    pub_key: pub_key.clone()
                });
            }
        }
    }
}

// Effectful action with service interaction
impl LedgerEffectfulAction {
    pub fn effects<S: LedgerService>(&self, _: &ActionMeta, store: &mut Store<S>) {
        match self {
            LedgerEffectfulAction::Write { request } => {
                store.service.write_ledger(request.clone());
            }
        }
    }
}
```

### Old Architecture Style (Transition Frontier)

The transition frontier still uses the original Redux pattern:

- **Reducers** only update state (no action dispatching)
- **Effects** handle all follow-up action dispatching after state changes
- Separate reducer and effects functions

This pattern matches traditional Redux but creates challenges in following the
flow since state updates and the resulting next actions are separated across
different files. The new style was introduced to improve code locality and make
the execution flow easier to follow.

### Callbacks Pattern

Callbacks enable dynamic action composition, allowing callers to specify
different flows after completion of the same underlying action. This pattern
solves several architectural problems:

**Before Callbacks:**

- All action flows were static and hardcoded
- Same actions needed to be duplicated for different completion flows
- Components were tightly coupled since actions had fixed next steps
- Adding new use cases required modifying existing actions

**With Callbacks:**

- Callers can reuse the same action with different completion behaviors
- Reduces component coupling by making actions more generic
- Eliminates action duplication across different contexts
- Easy to extend with new flows without modifying existing code

```rust
// Same action, different completion flows based on caller context
dispatcher.push(SnarkBlockVerifyAction::Init {
    req_id,
    block: block.clone(),
    on_success: redux::callback!(
        on_verify_success(hash: BlockHash) -> Action {
            ConsensusAction::BlockVerifySuccess { hash }  // Flow for consensus
        }
    ),
    on_error: redux::callback!(
        on_verify_error((hash: BlockHash, error: Error)) -> Action {
            ConsensusAction::BlockVerifyError { hash, error }
        }
    ),
});

// Same verification action, but different completion flow for RPC context
dispatcher.push(SnarkBlockVerifyAction::Init {
    req_id,
    block: block.clone(),
    on_success: redux::callback!(
        on_rpc_verify_success(hash: BlockHash) -> Action {
            RpcAction::BlockVerifyResponse { hash, success: true }  // Flow for RPC
        }
    ),
    on_error: redux::callback!(
        on_rpc_verify_error((hash: BlockHash, error: Error)) -> Action {
            RpcAction::BlockVerifyResponse { hash, success: false, error }
        }
    ),
});
```

### Directory Structure

Each major component follows a consistent pattern:

```
component/
├── component_state.rs         # State definition
├── component_actions.rs       # Stateful action types
├── component_reducer.rs       # State transitions + dispatching
└── component_effectful/       # Effectful actions
    ├── component_effectful_actions.rs
    ├── component_effectful_effects.rs
    └── component_service.rs   # Service interface
```

## Testing & Debugging

Testing benefits from the deterministic execution model:

### Testing Approaches

1. **Scenarios** - Specific network setups testing behaviors
2. **Simulator** - Multi-node controlled environments
3. **Fuzz Testing** - Random inputs finding edge cases
4. **Differential Fuzz Testing** - Comparing ledger implementation against the
   original OCaml version
5. **Invariant Checking** - Ensuring state consistency

### Debugging Features

1. **State Recording** - All inputs can be recorded
2. **Replay Capability** - Reproduce exact execution
3. **State Inspection** - Direct state examination in tests
4. **Deterministic Behavior** - Same inputs = same outputs

### Key Testing Properties

- **Determinism** - Predictable state transitions
- **Isolation** - State logic testable without services
- **Composability** - Complex scenarios from simple actions
- **Observability** - Full state visibility

## Development Guidelines

### Understanding the Codebase

1. **Start with State** - State definitions reveal the flow
2. **Follow Actions** - Stateful vs effectful distinction
3. **Check Enabling Conditions** - Understand validity rules
4. **Trace Callbacks** - See component interactions

### Adding New Features

1. **Design State First** - State should represent the flow
2. **Categorize Actions** - Stateful or effectful?
3. **Strict Enabling Conditions** - Prevent invalid states
4. **Use Callbacks** - For decoupled responses
5. **Keep Effects Thin** - Only service calls

### Best Practices

1. **State Represents Flow** - Make state self-documenting
2. **Actions Match Transitions** - Consistent naming conventions
3. **Reducers Handle Logic** - State updates + dispatching
4. **Effects Only Call Services** - No business logic
5. **Services Stay Minimal** - I/O and computation only

### Common Patterns

1. **Async Operations** - Effectful action → Service → Event → New action
   dispatch
2. **State Machines** - Enum variants representing stages
3. **Timeouts** - CheckTimeouts action triggers checks
4. **Error States** - Explicit error variants in state

### Architecture Evolution

The state machine components have been transitioning from old to new style:

- **New Style**: Unified reducers, thin effects, callbacks - most components
  have been migrated
- **Old Style**: Separate reducers/effects - transition frontier still uses this
  pattern
- **Migration Path**: State machine components updated incrementally

For detailed migration instructions, see
[ARCHITECTURE.md](../../ARCHITECTURE.md).

## Communication Patterns

The architecture provides several patterns for components to communicate while
maintaining decoupling and predictability.

### Direct Action Dispatching

Components can dispatch actions to trigger behavior in other components. This is
the primary pattern for synchronous communication.

**Example: Ledger to Block Producer Communication**

```rust
// From node/src/ledger/read/ledger_read_reducer.rs
// After receiving delegator table, notify block producer
match table {
    None => {
        dispatcher.push(
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                delegator_table: Default::default(),
            },
        );
    }
    Some(table) => {
        dispatcher.push(
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                delegator_table: table.into(),
            },
        );
    }
}
```

**Example: P2P Best Tip Propagation**

```rust
// From p2p/src/channels/best_tip/p2p_channels_best_tip_reducer.rs
// When best tip is received, update peer state
dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
```

### Callback Pattern

Components can register callbacks that get invoked when asynchronous operations
complete. This enables loose coupling between components.

**Example: P2P Channel Initialization**

```rust
// From p2p/src/channels/best_tip/p2p_channels_best_tip_reducer.rs
dispatcher.push(P2pChannelsEffectfulAction::InitChannel {
    peer_id,
    id: ChannelId::BestTipPropagation,
    on_success: redux::callback!(
        on_best_tip_channel_init(peer_id: PeerId) -> crate::P2pAction {
            P2pChannelsBestTipAction::Pending { peer_id }
        }
    ),
});
```

**Example: Transaction Pool Account Fetching**

```rust
// From node/src/transaction_pool/transaction_pool_reducer.rs
dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
    account_ids,
    ledger_hash: best_tip_hash.clone(),
    on_result: callback!(
        fetch_to_verify((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>, from_source: TransactionPoolMessageSource))
        -> crate::Action {
            TransactionPoolAction::StartVerifyWithAccounts { accounts, pending_id: id.unwrap(), from_source }
        }
    ),
    pending_id: Some(pending_id),
    from_source: *from_source,
});
```

### Event Source Pattern

Services communicate results back through events that get converted to actions.
The event source acts as the bridge between the async service world and the
synchronous state machine.

**Note:** Currently, all event handling is centralized in
`node/src/event_source/`. The architectural intention is to eventually
distribute this logic across the individual effectful state machines that care
about specific events, making the system more modular and maintainable.

**Example: Service Event Processing**

```rust
// From node/src/event_source/event_source_effects.rs
Event::Ledger(event) => match event {
    LedgerEvent::Write(response) => {
        store.dispatch(LedgerWriteAction::Success { response });
    }
    LedgerEvent::Read(id, response) => {
        store.dispatch(LedgerReadAction::Success { id, response });
    }
},
Event::Snark(event) => match event {
    SnarkEvent::BlockVerify(req_id, result) => match result {
        Err(error) => {
            store.dispatch(SnarkBlockVerifyAction::Error { req_id, error });
        }
        Ok(()) => {
            store.dispatch(SnarkBlockVerifyAction::Success { req_id });
        }
    },
}
```

### State Callbacks Pattern

Components can expose callbacks in their state that other components can
register to. This enables dynamic subscription to events.

**Example: P2P RPC Response Handling**

```rust
// From p2p/src/channels/rpc/p2p_channels_rpc_reducer.rs
let (dispatcher, state) = state_context.into_dispatcher_and_state();
let p2p_state: &P2pState = state.substate()?;

// Notify interested components about RPC response
if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_rpc_response_received {
    dispatcher.push_callback(callback.clone(), (peer_id, rpc_id, response));
}

// Handle timeout notifications
if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_rpc_timeout {
    dispatcher.push_callback(callback.clone(), (peer_id, id));
}
```

### Service Request with Callbacks

Components can make service requests and provide callbacks for handling both
success and error cases.

**Example: SNARK Verification Request**

```rust
// From node/src/transaction_pool/transaction_pool_reducer.rs
dispatcher.push(SnarkUserCommandVerifyAction::Init {
    req_id,
    commands: verifiable,
    from_source: *from_source,
    on_success: callback!(
        on_snark_user_command_verify_success(
            (req_id: SnarkUserCommandVerifyId, valids: Vec<valid::UserCommand>, from_source: TransactionPoolMessageSource)
        ) -> crate::Action {
            TransactionPoolAction::VerifySuccess {
                valids,
                from_source,
            }
        }
    ),
    on_error: callback!(
        on_snark_user_command_verify_error(
            (req_id: SnarkUserCommandVerifyId, errors: Vec<String>)
        ) -> crate::Action {
            TransactionPoolAction::VerifyError { errors }
        }
    )
});
```

### State Machine Lifecycle

#### Initialization

```
Main Node Init ──> Subsystem Creation ──> Service Spawning ──> Ready State
```

#### Action Processing

```
Event ──> Action Queue ──> Next Action ──> Enabling Check ──┐
              ▲                                │            │
              │                                │            ▼
              │                                │        Rejected
              │                                ▼
              │                            Reducer
              │                                │
              │                    ┌───────────┴───────────┐
              │                    │                       │
              │                    ▼                       ▼
              │              State Update          0+ Effectful Actions
              │                    │                       │
              │                    ▼                       ▼
              └──────── 0+ Stateful Actions        Service Calls
                                                          │
                                                          ▼
              Queue Empty ──> Listen for Events <─── Result Events
```

#### Effect Handling

```
Effectful Action ──> Service Call ──> Service Thread ──> Processing ──> Event
                                                                         │
                                                                         ▼
                                                                  Action Queue
```

### Mental Model

When working with this architecture, shift from imperative to declarative
thinking:

**State-First Design:**

- State enums represent the flow: `Idle → Pending → Success/Error`
- Actions represent transitions: "what event happened?" not "what should I do?"
- Reducers answer two questions:
  1. "Given this state and event, what's the new state?"
  2. "What are all possible next steps from here?"

**Reducer Orchestration:**

- Reducers update state AND dispatch multiple potential next actions
- Enabling conditions act as gates - only actions valid for the current state
  proceed
- This creates a branching execution where reducers propose paths and conditions
  filter them

**Action Classification:**

- **Stateful**: Updates state, dispatches other actions (business logic)
- **Effectful**: Calls services, never updates state directly (I/O boundary)
- **Events**: External inputs wrapped in actions (deterministic replay)

**Async Operations Pattern:**

```
1. Dispatch Effectful Action → 2. Service processes → 3. Event generated → 4. Action dispatched
```

**Debugging Mental Model:**

- Logs show the exact sequence of actions - trace execution flow
- State inspection reveals current system state at any moment
- Actions can be recorded for deterministic replay (when enabled)
- Common bugs: missing enabling conditions, incorrect state transitions

**Common Mental Shift:** Instead of "call API then update state", think
"dispatch action, let reducer update state and propose next actions, enabling
conditions filter valid paths based on current state, services report back via
events that trigger new actions".

The architecture may feel unusual initially, but its benefits in correctness,
testability, and debuggability make it powerful for building reliable
distributed systems.

# OpenMina State Machine Structure

This document maps out the complete hierarchy and organization of state machines
in OpenMina, showing how dozens of state machines are structured and their
relationships.

> **Prerequisites**: Read
> [Architecture Walkthrough](architecture-walkthrough.md) first to understand
> the Redux pattern and core concepts. **Next Steps**: After understanding the
> structure, see [State Machine Patterns](state-machine-patterns.md) for common
> patterns, then [Project Organization](organization.md) to navigate the
> codebase.

## Architectural Layers

### 1. Top-Level Orchestration

```
┌─────────────────────────────────────────────────────────────┐
│                    Main Node State Machine                  │
│                       (node/src/)                           │
├─────────────────────────────────────────────────────────────┤
│ • Orchestrates all subsystems                               │
│ • Routes actions between components                         │
│ • Manages node lifecycle                                    │
│ • Coordinates P2P, consensus, storage, and RPC              │
└─────────────────────────────────────────────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                      ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   P2P System    │  │  SNARK System   │  │ Node Subsystems │
│  (p2p/src/)     │  │  (snark/src/)   │  │  (node/src/*)   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

### 2. Core Systems

#### P2P State Machine (`p2p/src/`)

Manages all peer-to-peer networking functionality through two distinct network
layers:

- **Connection Management**: Handles incoming/outgoing peer connections
- **Dual Network Support**:
  - libp2p protocols (Kademlia, PubSub, etc.) for native nodes
  - WebRTC-based network for webnodes with different design patterns
- **Message Routing**: Routes protocol messages between peers via channel
  abstractions
- **Peer Discovery**: Maintains peer registry and discovery mechanisms

#### SNARK State Machine (`snark/src/`)

Handles proof verification:

- **Block Verification**: Verifies block proofs
- **Work Verification**: Verifies SNARK work proofs from workers
- **User Command Verification**: Verifies transaction proofs and zkApp proofs

### 3. Node Subsystems

#### Block Producer (`node/src/block_producer/`)

Manages block production for validator nodes:

- **VRF Evaluator**: Determines slot leadership eligibility
- **Block Construction**: Assembles blocks when node wins slots
- **Transaction Selection**: Chooses transactions from mempool

#### Transition Frontier (`node/src/transition_frontier/`)

Core consensus and blockchain state management:

- **Block Processing**: Validates and accepts new blocks
- **Chain Selection**: Handles reorganizations and best tip selection
- **Synchronization**: Downloads missing blocks and ledger data
- **Genesis**: Initializes blockchain from genesis configuration

_Note: This subsystem still uses the old-style state machine pattern and is
scheduled for migration._

#### Transaction Pool (`node/src/transaction_pool/`)

Maintains mempool of pending transactions:

- **Validation**: Pre-validates transactions before inclusion
- **Prioritization**: Orders by fee for block inclusion
- **Eviction**: Removes invalid/expired transactions
- **Propagation**: Shares transactions with peers

#### SNARK Pool (`node/src/snark_pool/`)

Manages pool of SNARK work proofs:

- **Work Collection**: Receives proofs from external workers
- **Validation**: Verifies work correctness
- **Pricing**: Manages work fee market
- **Distribution**: Provides work for block production

#### Ledger (`node/src/ledger/`)

Manages blockchain account state:

- **Read Operations**: Concurrent account queries
- **Write Operations**: Atomic state updates from blocks
- **Merkle Proofs**: Generates cryptographic proofs

#### RPC (`node/src/rpc/`)

External API interface:

- **Request Handling**: Processes GraphQL/REST queries
- **Response Formatting**: Serializes node data
- **Client Management**: Handles WebSocket subscriptions

#### External SNARK Worker (`node/src/external_snark_worker/`)

Manages external proof computation:

- **Process Management**: Spawns/monitors worker processes
- **Work Distribution**: Assigns proof tasks
- **Result Collection**: Gathers completed proofs

#### Watched Accounts (`node/src/watched_accounts/`)

Account monitoring system:

- **Registration**: Tracks specific accounts
- **Event Detection**: Monitors balance changes
- **Notifications**: Emits events for account updates

## Key Interaction Flows

### 1. Block Production Flow

```
Transaction Pool ──┐
                   ├──> Block Producer ──> New Block ──> P2P Broadcast
SNARK Pool ────────┘                                       │
                                                           ▼
                                                    Transition Frontier
```

1. **Block Producer checks eligibility** → VRF evaluation determines slot
   leadership
2. **Gather transactions** → Pulls from Transaction Pool based on fees
3. **Include SNARK work** → Selects required proofs from SNARK Pool
4. **Construct block** → Assembles block with transactions and proofs
5. **Broadcast block** → P2P system propagates to peers
6. **Update local state** → Transition Frontier processes own block

### 2. Block Reception Flow

```
P2P Network ──> Block Reception ──> SNARK Verification ──┐
                                                         ├──> Transition Frontier
                                                         │         │
                                                         │         ▼
                                                         └──> Ledger Updates
```

1. **P2P receives block** → Channels process incoming data
2. **Transition frontier notified** → `BlockReceived` action dispatched
3. **Reducer updates state** → Stores block, dispatches verification
4. **SNARK verification initiated** → Effectful action calls service
5. **Service processes proof** → Async verification runs
6. **Event returned** → Success/failure event dispatched
7. **Callback triggered** → Original callback action executed
8. **Block applied if valid** → State machine updates blockchain

### 3. Transaction Flow

```
RPC/P2P ──> Transaction Pool ──> Validation ──┐
                                              ├──> P2P Propagation
                                              └──> Block Inclusion
```

1. **P2P broadcasts** → Via Gossipsub protocol
2. **Pool receives transaction** → Stateful action triggered
3. **Validation requested** → Effectful action to ledger service
4. **Service validates** → Checks against current state
5. **Result event** → Valid/invalid status returned
6. **Pool updated** → Transaction added if valid
7. **Propagate to peers** → Valid transactions shared via P2P

### 4. SNARK Work Flow

```
External Worker ──> SNARK Pool ──> Validation ──┐
                                                ├──> P2P Distribution
                                                └──> Block Production
```

1. **Work received via P2P** → Gossip message processed
2. **Candidate created** → State machine tracks verification
3. **Batch verification** → Multiple proofs queued together
4. **Service verifies** → Async proof checking
5. **Results returned** → Events for each verification
6. **Pool updated** → Valid work added to pool
7. **Distribution** → Share with peers and make available for blocks

### P2P Subsystem Architecture

The P2P system contains multiple specialized state machines:

#### Channel State Machines (`p2p/src/channels/`)

Protocol-specific communication handlers:

- **Best Tip**: Propagates chain head updates
- **Transaction**: Gossips pending transactions
- **SNARK**: Distributes SNARK work
- **RPC**: Handles peer-to-peer queries
- **Streaming RPC**: Manages long-lived data streams
- **Signaling**: WebRTC connection establishment

#### Network State Machines (`p2p/src/network/`)

Low-level protocol implementations:

- **Kademlia**: DHT for peer discovery
  - Bootstrap: Initial network joining
  - Request: Query processing
  - Stream: Protocol communication
- **PubSub**: Gossip protocol for broadcasts
- **Identify**: Peer information exchange
- **Yamux**: Stream multiplexing
- **Noise**: Encryption protocol
- **Select**: Protocol negotiation

#### Connection Management (`p2p/src/connection/`)

- **Incoming**: Handles inbound connections
- **Outgoing**: Initiates outbound connections
- **Disconnection**: Graceful disconnect handling

### 5. Peer Connection Flow

Establishing peer connections differs by network type:

**libp2p-based connections (native and OCaml nodes):**

1. **Connection initiated** → Outgoing or incoming TCP connection
2. **Security established** → Noise handshake effectful actions
3. **Multiplexing setup** → Yamux stream creation
4. **Identify exchanged** → Peer capabilities shared
5. **Ready state reached** → Peer available for messaging

**WebRTC-based connections (webnodes):**

1. **Signaling initiated** → WebRTC connection establishment
2. **Direct connection** → Peer-to-peer WebRTC channel
3. **Channel ready** → Direct communication available
4. **Ready state reached** → Peer available for messaging

## State Access Control: Substate System

OpenMina uses a substate system (`core/src/substate.rs`) to decouple reducers
from the global state representation. This abstraction ensures components don't
depend on the exact structure of the global state:

### How It Works

Reducers receive `Substate` values that provide access to specific state
portions:

- **Decoupling**: Components work with their own state types without knowing the
  global state structure
- **Modularity**: State machine components can be moved to their own crates in
  the future if necessary
- **Type safety**: Access boundaries are enforced at compile time
- **Flexibility**: Global state structure can change without updating all
  reducers

### Phase Separation Enforcement

The `Substate` system enforces the two-phase reducer pattern through its API
design:

1. **Phase 1 - State Updates**: `state_context.get_substate_mut()` provides
   mutable access to state
2. **Phase 2 - Action Dispatching**: `state_context.into_dispatcher()` consumes
   the context and returns a dispatcher
   - Use `into_dispatcher()` for simple action dispatching
   - Use `into_dispatcher_and_state()` when you need read-only access to global
     state for coordination

This design makes it impossible to mix state updates and action dispatching:

- Once you call `into_dispatcher()`, you can no longer access mutable state
- The dispatcher can only dispatch actions, not modify state
- The type system enforces this separation at compile time

```rust
// Phase 1: State updates only
let Ok(state) = state_context.get_substate_mut() else { return };
state.field = new_value; // ✓ Allowed

// Phase 2: Action dispatching only
let dispatcher = state_context.into_dispatcher();
// Or for global state access:
// let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
dispatcher.push(SomeAction { ... }); // ✓ Allowed
// state.field = other_value; // ✗ Compiler error - state no longer accessible
```

### Implementation

Substate accesses are defined in `node/src/state.rs` using the
`impl_substate_access!` macro. For example:

- `impl_substate_access!(State, SnarkState, snark)` - Access to SNARK subsystem
  state
- `impl_substate_access!(State, TransitionFrontierState, transition_frontier)` -
  Access to blockchain state
- Custom implementations for conditional access (e.g., P2P state only available
  when initialized)

This pattern is fundamental to maintaining modularity in the Redux-style
architecture and enables future refactoring without breaking existing
components.

## Migration Note

The Transition Frontier subsystem (`node/src/transition_frontier/`) currently
uses an older state machine pattern and is scheduled for migration to the new
architecture style.

For migration instructions, see [ARCHITECTURE.md](../../ARCHITECTURE.md).

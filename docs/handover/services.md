# OpenMina Services

This document provides a roadmap to services in the OpenMina system. Services
handle external I/O, heavy computations, and asynchronous operations, keeping
the state machine deterministic and pure.

## Architecture Overview

Services isolate non-deterministic operations from the Redux state machine:

- **State machines** handle business logic and dispatch effectful actions
- **Services** handle "outside world" interactions (I/O, crypto, networking)
- **Events** carry results back from services to state machines
- **Threading** enables CPU-intensive work without blocking state machine

For architectural details, see
[`architecture-walkthrough.md`](architecture-walkthrough.md).

## Service Organization

Services follow a consistent pattern:

- **Trait** - Interface defined in `*_effectful_service.rs` files
- **Implementation** - Platform-specific implementations in:
  - `node/native/src/service/` - Native implementations
  - `node/web/src/` - WASM implementations
  - `p2p/src/service_impl/` - P2P implementations

## Core System Services

### EventSourceService

**Trait**: `node/src/event_source/event_source_service.rs`  
**Implementation**: `node/native/src/service/mod.rs` (part of `NodeService`)

Central event aggregation service that collects events from all services,
batches them, and routes them to the state machine. Provides the bridge between
async service world and synchronous state machine.

**Usage Pattern:** Services send events → EventSource batches them → Main loop
processes via `EventSourceAction::ProcessEvents`

### TimeService

**Location**: `node/common/src/service/service.rs` (part of `NodeService`)

Provides time abstraction for the entire system, enabling deterministic replay.

**Key Concepts:**

- Abstracts system time for deterministic execution
- Normal mode returns actual system time, replay mode returns recorded
  timestamps
- Critical for slot calculations, VRF evaluation, and block production timing
- All actions receive timestamps from TimeService

**Why it matters:** Makes non-deterministic time access deterministic, enabling
perfect reproduction of execution sequences and debugging of time-sensitive
consensus issues.

### LedgerService

**Trait**: `node/src/ledger/ledger_service.rs`  
**Implementation**: `node/src/ledger/ledger_manager.rs` (dedicated thread)

Provides interface to the LedgerManager for all ledger operations.

**Threading Model:**

- Dedicated "ledger-manager" thread for all operations
- Worker threads spawned for heavy computations
- Async communication via event-based responses

**Key Operations:**

- **Read**: Account queries, merkle tree lookups, scan state info
- **Write**: Block application, staged ledger operations, commits
- **Storage**: Manages snarked ledgers, staged ledgers, and sync state

**Note**: Contains deprecated synchronous methods (`get_accounts()`,
`get_mask()`) that should not be used in new code.

## P2P Networking Services

### P2pService

**Trait**: `p2p/src/p2p_service.rs`  
**Implementation**: `p2p/src/service_impl/` (multiple backends)

Composite service managing all peer-to-peer networking operations.

**Core Sub-services:**

- **P2pConnectionService** - WebRTC connection establishment and authentication
- **P2pDisconnectionService** - Peer disconnection handling
- **P2pChannelsService** - Channel communication and message
  encryption/decryption

**Extended Sub-services (with libp2p):**

- **P2pMioService** - Low-level network I/O and socket management
- **P2pCryptoService** - Cryptographic operations
- **P2pNetworkService** - Network utilities (DNS, IP detection)

**Architecture Notes:**

- Each peer runs in dedicated async task
- WebRTC: SDP exchange → authentication → data channels
- Trait-based composition enables different backends
- Services handle only I/O, never business logic

## SNARK Verification Services

All SNARK verification services delegate to the `ledger` crate for actual
cryptographic operations.

### SnarkBlockVerifyService

**Trait**: `snark/src/block_verify_effectful/snark_block_verify_service.rs`  
**Implementation**: `node/common/src/service/snarks.rs`

Verifies block proofs using dedicated "block_proof_verifier" thread. Called from
transition frontier when blocks need proof verification.

### SnarkUserCommandVerifyService

**Trait**:
`snark/src/user_command_verify_effectful/snark_user_command_verify_service.rs`  
**Implementation**: `node/common/src/service/snarks.rs`

Verifies user command signatures using Rayon thread pool for parallel
processing. Called from transaction pool for signature validation.

### SnarkWorkVerifyService

**Trait**: `snark/src/work_verify_effectful/snark_work_verify_service.rs`  
**Implementation**: `node/common/src/service/snarks.rs`

Verifies SNARK work submissions (transaction and zkApp proofs) using Rayon
thread pool. Called from SNARK pool when evaluating work submissions.

### ExternalSnarkWorkerService

**Trait**:
`node/src/external_snark_worker_effectful/external_snark_worker_service.rs`  
**Implementation**: `node/common/src/service/snark_worker.rs`

Manages external SNARK worker process for scan state SNARK work production.

**Key Operations:**

- Start/stop worker with fee configuration
- Submit work specifications for proof generation
- Generates transaction and zkApp proofs via dedicated "snark_worker" thread

**Usage:** Called exclusively from SNARK pool for scan state work production.

## Block Production Services

### BlockProducerService

**Trait**:
`node/src/block_producer_effectful/block_producer_effectful_service.rs`  
**Implementation**: `node/common/src/service/block_producer/mod.rs`

Provides block proof generation using dedicated "openmina_block_prover" thread.

**Key Operations:**

- Returns cached block prover instances
- Generates block proofs with blockchain state input
- Provides secure access to producer's secret key
- Failed proofs dump encrypted debug data to disk

### BlockProducerVrfEvaluatorService

**Trait**:
`node/src/block_producer_effectful/vrf_evaluator_effectful/block_producer_vrf_evaluator_effectful_service.rs`  
**Implementation**:
`node/common/src/service/block_producer/vrf_evaluator.rs`

Evaluates VRF for slot leadership determination using dedicated
"openmina_vrf_evaluator" thread. Receives epoch seed, delegator table, and slot
info to determine if node won the slot based on stake distribution.

## Pool Management Services

### SnarkPoolService

**Trait**: `node/src/snark_pool/snark_pool_service.rs`  
**Implementation**: `node/common/src/service/service.rs` (part of `NodeService`)

Provides randomization for SNARK job selection when using random snarker
strategy. Isolates non-deterministic random selection from the deterministic
state machine.

### VerifyUserCommandsService (Dead Code)

**Location**: `node/src/transaction_pool/transaction_pool_service.rs`

Unused trait with no implementations. Transaction verification is actually
handled by `SnarkUserCommandVerifyService`. Should be removed.

## State Synchronization Services

### TransitionFrontierGenesisService

**Trait**:
`node/src/transition_frontier/genesis_effectful/transition_frontier_genesis_service.rs`  
**Implementation**:
`node/common/src/service/service.rs` (part of `NodeService`)

Manages genesis configuration loading and ledger initialization.

### TransitionFrontierSyncLedgerSnarkedService

**Trait**:
`node/src/transition_frontier/sync/ledger/snarked/transition_frontier_sync_ledger_snarked_service.rs`  
**Implementation**:
Generic implementation delegating to `LedgerService`

Handles snarked ledger operations during blockchain synchronization:

- Merkle tree operations (child hash retrieval, hash computation)
- Ledger management (copying for sync, account population)
- All operations delegate to LedgerManager for thread-safe access

## External Integration Services

### RpcService

**Trait**: `node/src/rpc_effectful/rpc_service.rs`  
**Implementation**: `node/common/src/service/rpc/mod.rs` (part of `NodeService`)

Manages RPC response delivery to external clients through channel-based
communication. Contains 30+ `respond_*` methods for different RPC operations.

**Response Categories:** State, Status, P2P, SNARK Pool, Transactions, Ledger,
Consensus

**Key Pattern:** Service only handles response delivery - all RPC logic is in
the state machine. Uses unique `RpcId` for request/response correlation.

### ArchiveService

**Trait**: `node/src/transition_frontier/archive/archive_service.rs`  
**Implementation**: `node/common/src/service/archive/mod.rs`

Provides asynchronous block persistence to external storage systems.

**Storage Backends:**

- AWS S3 - JSON storage with configurable bucket/region
- Google Cloud Platform - Cloud storage integration
- Local Filesystem - Direct file system storage
- Archive Process - RPC communication with external archiver

Uses dedicated thread with Tokio runtime for async operations. Implements retry
logic (5 attempts) for failed uploads. Called exclusively during block
application.

## Service Lifecycle

Services are created via `NodeServiceCommonBuilder`, configured based on node
type, and connected via event channels. Dedicated threads are spawned for
CPU-intensive services.

**Runtime Flow:** State machine dispatches effectful actions → Effects call
service methods → Services perform async operations → Results sent back via
events

## Service Implementation Patterns

**Implementation Locations:**

- **Node Services** (`node/native/src/service/`) - Most services, often
  delegating to specialized crates
- **P2P Services** (`p2p/src/service_impl/`) - Multiple backends (libp2p,
  WebRTC)
- **Web/WASM Services** (`node/web/src/`) - Browser-compatible implementations

**Threading Patterns:**

- Some services use dedicated threads (LedgerManager, VRF evaluator, SNARK
  workers)
- Others use Rayon thread pool or run on main thread
- WASM uses web workers instead of threads

**Communication Patterns:**

- State Machine → Service: Via effectful actions
- Service → State Machine: Via events with request IDs
- Service → Service: Not allowed - all coordination through state machine

This architecture provides clean separation between deterministic state
management and non-deterministic external operations.

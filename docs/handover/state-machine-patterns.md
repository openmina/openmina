# State Machine Patterns in OpenMina

This guide describes the patterns we use for state machines in OpenMina and when
to apply each one. These patterns grew organically from different contributors
over time, and there may be opportunities for normalization.

> **Prerequisites**: Read
> [Architecture Walkthrough](architecture-walkthrough.md) and
> [State Machine Structure](state-machine-structure.md) first. **Related**: See
> [State Machine Development Guide](state-machine-development-guide.md) for
> implementation details.

## Our State Machine Patterns

OpenMina contains dozens of state machines that use several distinct patterns:

1. **Pure Lifecycle Pattern** - Simple async operations
2. **Sequential Lifecycle Pattern** - Multi-phase operations with state
   accumulation
3. **Connection Lifecycle Pattern** - Complex protocol negotiations
4. **Iterative Process Pattern** - Long-running processes with stepping
5. **Worker State Machine Pattern** - Process management with operational loops
6. **Hybrid Patterns** - Complex domain workflows with embedded patterns

**Important**: Many actions exist for **debugging and testing granularity**
rather than just async operations. This enables precise state tracking, better
simulator tests, and detailed logging.

## Quick Reference

| Pattern              | Use For                     | Naming Convention                   | Example                  |
| -------------------- | --------------------------- | ----------------------------------- | ------------------------ |
| Pure Lifecycle       | Simple async operations     | `Init/Pending/Success/Error`        | SNARK verification       |
| Sequential Lifecycle | Multi-phase sync operations | `Phase1Pending/Phase1Success`       | Transition frontier sync |
| Connection Lifecycle | Network protocol handshakes | `Phase + Pending/Success`           | P2P connections          |
| Iterative Process    | Long-running computations   | `Begin/Continue/Finish/Interrupt`   | VRF epoch evaluation     |
| Worker State Machine | External process management | `Starting/Idle/Working/Ready/Error` | External SNARK worker    |
| Hybrid Pattern       | Complex domain workflows    | Mixed patterns as appropriate       | Block producer           |

## Common Patterns

### 1. Pure Lifecycle Pattern (Init → Pending → Success/Error)

**Use for**: Simple async operations that follow a clear lifecycle.

#### SNARK Verification (`snark/src/block_verify/`)

```rust
pub enum SnarkBlockVerifyAction {
    Init { /* block data */ },
    Pending { /* verification progress */ },
    Success { /* verification result */ },
    Error { /* verification error */ },
    Finish { /* cleanup */ },
}

pub enum SnarkBlockVerifyState {
    Init { /* ... */ },
    Pending { /* ... */ },
    Success { /* ... */ },
    Error { /* ... */ },
}
```

**Pattern**: Single async operation, clear linear flow, simple error handling.

#### Transaction Pool Candidate (`node/src/transaction_pool/candidate/`)

```rust
pub enum TransactionPoolCandidateAction {
    InfoReceived { /* transaction info */ },
    FetchInit { /* start fetching */ },
    FetchPending { /* fetch progress */ },
    FetchError { /* fetch failed */ },
    FetchSuccess { /* transaction fetched */ },
    VerifyPending { /* verification progress */ },
    VerifyError { /* verification failed */ },
    VerifySuccess { /* verification complete */ },
}

pub enum TransactionPoolCandidateState {
    InfoReceived { /* ... */ },
    FetchPending { /* ... */ },
    Received { /* ... */ },
    VerifyPending { /* ... */ },
    VerifyError { /* ... */ },
    VerifySuccess { /* ... */ },
}
```

**Pattern**: Multiple phases, each following lifecycle (fetch phase, then verify
phase).

### 2. Sequential Lifecycle Pattern

**Complex sync operations** with multiple sequential lifecycle phases.

#### Transition Frontier Sync (`node/src/transition_frontier/sync/`)

```rust
pub enum TransitionFrontierSyncState {
    Idle,
    Init { /* sync start */ },

    // Phase 1: Staking ledger sync
    StakingLedgerPending { /* ... */ },
    StakingLedgerSuccess { /* ... */ },

    // Phase 2: Next epoch ledger sync
    NextEpochLedgerPending { /* ... */ },
    NextEpochLedgerSuccess { /* ... */ },

    // Phase 3: Root ledger sync
    RootLedgerPending { /* ... */ },
    RootLedgerSuccess { /* ... */ },

    // Phase 4: Block sync
    BlocksPending { /* ... */ },
    BlocksSuccess { /* ... */ },

    // Phase 5: Commit
    CommitPending { /* ... */ },
    CommitSuccess { /* ... */ },

    Synced { /* final state */ },
}
```

**Pattern**: Multiple sequential phases, each with Pending → Success lifecycle.
Each Success state carries forward the data needed for the next phase.

### 3. Connection Lifecycle Pattern

**Network connections** that need complex handshake flows.

#### P2P Outgoing Connection (`p2p/src/connection/outgoing/`)

```rust
pub enum P2pConnectionOutgoingState {
    Init { /* connection parameters */ },

    // SDP creation phase
    OfferSdpCreatePending { /* ... */ },
    OfferSdpCreateSuccess { /* SDP created */ },

    // Offer phase
    OfferReady { /* offer ready */ },
    OfferSendSuccess { /* offer sent */ },

    // Answer phase
    AnswerRecvPending { /* waiting for answer */ },
    AnswerRecvSuccess { /* answer received */ },

    // Finalization phase
    FinalizePending { /* finalizing connection */ },
    FinalizeSuccess { /* connection established */ },

    // Terminal states
    Success { /* connected */ },
    Error { /* connection failed */ },
}
```

**Pattern**: Multiple phases with detailed intermediate states. Each phase has
its own success state that flows to the next phase.

### 4. Iterative Process Pattern

**Long-running processes** that execute in steps over time.

#### VRF Epoch Evaluation (`node/src/block_producer/vrf_evaluator/`)

```rust
pub enum BlockProducerVrfEvaluatorAction {
    // Process control
    BeginEpochEvaluation { /* start parameters */ },
    ContinueEpochEvaluation { /* step parameters */ },
    FinishEpochEvaluation { /* completion data */ },

    // Process interruption
    InterruptEpochEvaluation { reason: InterruptReason },

    // Sub-processes (also iterative)
    BeginDelegatorTableConstruction,
    FinalizeDelegatorTableConstruction { /* table data */ },

    // Individual evaluations
    EvaluateSlot { /* slot data */ },
    ProcessSlotEvaluationSuccess { /* evaluation result */ },
}
```

**Pattern**: Long-running process that:

- **Begins** with initialization
- **Continues** through multiple steps/iterations
- **Finishes** when complete or interrupted
- Can be **interrupted** and potentially resumed

**Key Insight**: Not all Begin/Finish pairs are lifecycle patterns - some
represent iterative processes.

### 5. Worker State Machine Pattern

**Worker processes** with lifecycle + operational states.

#### External SNARK Worker (`node/src/external_snark_worker/`)

```rust
pub enum ExternalSnarkWorkerState {
    None,
    Starting,        // Lifecycle: init

    // Operational states
    Idle,
    Working(SnarkWorkId, JobSummary),
    WorkReady(SnarkWorkId, SnarkWorkResult),
    WorkError(SnarkWorkId, ExternalSnarkWorkerWorkError),

    // Cancellation lifecycle
    Cancelling(SnarkWorkId),
    Cancelled(SnarkWorkId),

    // Shutdown lifecycle
    Killing,
    Error(ExternalSnarkWorkerError, bool),
}
```

**Pattern**: Initialization lifecycle → Operational loop (Idle → Working →
Ready/Error) → Shutdown lifecycle. Worker can be cancelled during operation.

### 6. Hybrid Lifecycle + Domain-Specific Patterns

**Complex business workflows** that embed lifecycle patterns within
domain-specific flows.

#### Block Producer (`node/src/block_producer/`)

```rust
pub enum BlockProducerAction {
    // Domain-specific workflow
    VrfEvaluator { /* VRF evaluation */ },
    WonSlotSearch { /* slot winning check */ },
    WonSlot { /* slot won */ },
    WonSlotWait { /* waiting for slot */ },

    // Lifecycle pattern for staged ledger diff creation
    StagedLedgerDiffCreateInit { /* diff creation start */ },
    StagedLedgerDiffCreatePending { /* diff creation progress */ },
    StagedLedgerDiffCreateSuccess { /* diff created */ },

    // Lifecycle pattern for block proving
    BlockProveInit { /* proof generation start */ },
    BlockProvePending { /* proof generation progress */ },
    BlockProveSuccess { /* proof generated */ },

    // Domain-specific completion
    BlockProduced { /* block completed */ },
    BlockInject { /* inject into network */ },
    BlockInjected { /* injection completed */ },
}

pub enum BlockProducerCurrentState {
    Idle { /* ... */ },
    WonSlot { /* ... */ },
    WonSlotWait { /* ... */ },
    StagedLedgerDiffCreatePending { /* ... */ },  // Lifecycle state
    StagedLedgerDiffCreateSuccess { /* ... */ },  // Lifecycle state
    BlockProvePending { /* ... */ },              // Lifecycle state
    BlockProveSuccess { /* ... */ },              // Lifecycle state
    Produced { /* ... */ },
    Injected { /* ... */ },
}
```

**Pattern**: Domain-specific workflow orchestrates multiple async operations,
each using lifecycle patterns internally.

## When to Use Each Pattern & Implementation Guidelines

### Use Pure Lifecycle Pattern When:

- ✅ **Single async operation** (SNARK verification, simple fetches)
- ✅ **Linear flow** with clear start → progress → completion
- ✅ **Simple error handling** (retry or abort)
- ✅ **No complex business logic** between states

**Implementation**:

```rust
// Good - consistent lifecycle naming
pub enum MyAction {
    FetchInit { /* ... */ },
    FetchPending { /* ... */ },
    FetchSuccess { /* ... */ },
    FetchError { /* ... */ },  // Always include error handling
}
```

### Use Sequential Lifecycle Pattern When:

- ✅ **Multiple phases** that must complete in order
- ✅ **Each phase** is an async operation with its own lifecycle
- ✅ **State accumulation** (each phase builds on previous results)
- ✅ **Complex sync operations** (ledger sync, blockchain sync)

### Use Connection Lifecycle Pattern When:

- ✅ **Network protocols** with handshake flows
- ✅ **Multiple negotiation phases** (SDP, offer, answer, finalize)
- ✅ **Detailed intermediate states** needed for debugging
- ✅ **Connection establishment** processes

### Use Iterative Process Pattern When:

- ✅ **Long-running computations** that need to be stepped
- ✅ **Interruptible processes** that can be paused/resumed
- ✅ **Progress tracking** through multiple iterations
- ✅ **Examples**: VRF evaluation, epoch processing, large computations

**Implementation**:

```rust
// Good - iterative process naming
pub enum MyAction {
    BeginComputation { /* ... */ },
    ContinueComputation { /* ... */ },
    FinishComputation { /* ... */ },
    InterruptComputation { /* ... */ },
}
```

### Use Worker State Machine Pattern When:

- ✅ **Worker processes** with start/stop lifecycle
- ✅ **Operational loop** (idle → working → result)
- ✅ **Cancellation support** during operation
- ✅ **External process management** (SNARK workers, external services)

### Use Hybrid Lifecycle + Domain Pattern When:

- ✅ **Complex business workflows** with embedded async operations
- ✅ **Domain-specific states** mixed with lifecycle operations
- ✅ **Multiple concerns** in one state machine
- ✅ **Examples**: Block production, transaction pool processing

**Implementation**:

```rust
// Group related lifecycle operations together
pub enum BlockProducerAction {
    // Domain workflow
    WonSlot { /* ... */ },
    WonSlotWait { /* ... */ },

    // Lifecycle group 1: Diff creation
    StagedLedgerDiffCreateInit { /* ... */ },
    StagedLedgerDiffCreatePending { /* ... */ },
    StagedLedgerDiffCreateSuccess { /* ... */ },

    // Lifecycle group 2: Block proving
    BlockProveInit { /* ... */ },
    BlockProvePending { /* ... */ },
    BlockProveSuccess { /* ... */ },

    // Domain completion
    BlockProduced { /* ... */ },
}
```

## Common Anti-Patterns to Avoid

### 1. Missing Error Handling

```rust
// Bad - no error action
pub enum MyAction {
    Init,
    Pending,
    Success,  // What happens if this fails?
}

// Good - complete error handling
pub enum MyAction {
    Init,
    Pending,
    Success,
    Error { error: String, should_retry: bool },
}
```

### 2. Inconsistent Naming

```rust
// Bad - mixing patterns
pub enum MyAction {
    BeginFetch,     // Iterative style
    FetchPending,   // Lifecycle style
    FinalizeFetch,  // Inconsistent
}

// Good - consistent lifecycle
pub enum MyAction {
    FetchInit,
    FetchPending,
    FetchSuccess,
    FetchError,
}
```

### 3. Overly Complex State Hierarchies

```rust
// Avoid - unnecessarily complex
pub enum BadAction {
    PreInit { /* ... */ },
    Init { /* ... */ },
    PostInit { /* ... */ },
    PrePending { /* ... */ },
    Pending { /* ... */ },
    PostPending { /* ... */ },
}
```

### 4. Unclear State Purpose

```rust
// Bad - unclear state purpose
pub enum MyState {
    StateA { /* what does this do? */ },
    StateB { /* when does this happen? */ },
}

// Good - clear state meaning
pub enum MyState {
    Init { /* initialization data */ },
    Pending { /* async operation in progress */ },
    Success { /* operation completed */ },
}
```

## Action Granularity for Debugging

**Many actions exist for debugging/testing granularity, not async necessity:**

```rust
// Block Producer - granular actions for debugging
pub enum BlockProducerAction {
    // These provide debugging visibility into async operation
    StagedLedgerDiffCreateInit,     // Debugging: marks start
    StagedLedgerDiffCreatePending,  // Debugging: shows progress
    StagedLedgerDiffCreateSuccess,  // Debugging: marks completion

    // Only the Init triggers actual async work
    // Pending/Success are for state tracking and logging
}
```

**Benefits:**

- **Simulator tests** can verify exact state transitions
- **Invariant checker** can validate state at each step
- **Logging** shows detailed progress for debugging
- **Monitoring** can track operation phases

## Known Issues and Improvement Opportunities

### Missing Error Handling

**Block Producer lacks error actions for critical operations:**

```rust
// Current - missing error handling
pub enum BlockProducerAction {
    BlockProveInit,
    BlockProvePending,
    BlockProveSuccess { proof: Arc<MinaBaseProofStableV2> },
    // MISSING: BlockProveError - what happens when proof fails?
}
```

**Should add:**

```rust
pub enum BlockProducerAction {
    BlockProveInit,
    BlockProvePending,
    BlockProveSuccess { proof: Arc<MinaBaseProofStableV2> },
    BlockProveError {
        error: String,
        retry_count: u32,
        should_retry: bool,
    },
}
```

### Consider Normalization When:

- ⚠️ **Inconsistent naming** for similar operations
- ⚠️ **Missing Error actions** for operations that can fail
- ⚠️ **Begin/Finalize** used for simple async instead of **Init/Success**
- ⚠️ **Mix of patterns** within same state machine without clear reason

**Note**: `Begin/Continue/Finish` is correct for iterative processes, but
`Init/Pending/Success` is better for simple async operations.

## Best Practices for New State Machines

1. **Choose the Right Pattern**: Match pattern to problem complexity
2. **Always Include Error Handling**: Every async operation needs Error actions
3. **Design for Debugging**: Use granular actions for state visibility
4. **Use Consistent Naming**: Follow established patterns within your domain
5. **Carry State Forward**: Each phase should build on previous results
6. **Group Related Operations**: Keep lifecycle operations together

### Naming Conventions:

- **Lifecycle**: `Init/Pending/Success/Error`
- **Iterative**: `Begin/Continue/Finish/Interrupt`
- **Worker**: `Starting/Idle/Working/Ready/Error`
- Don't mix patterns without clear reason

## Conclusion

These state machine patterns have evolved organically to serve different needs,
from simple async operations to complex domain workflows. The diversity enables
appropriate pattern selection for each problem domain, while granular actions
provide excellent debugging capabilities.

When implementing new state machines, follow these established patterns to
maintain consistency and leverage the debugging infrastructure. The key is
matching the pattern to the problem complexity rather than forcing simple
problems into complex patterns.

For implementation details and migration guidance, see
[ARCHITECTURE.md](../../ARCHITECTURE.md).

# SNARK Pool State Machine

Manages the distributed pool of SNARK work jobs required for blockchain
compression through coordination of local workers, P2P networking, and
competitive work selection.

## Purpose

- Maintains distributed pool of available SNARK computation jobs from scan state
- Coordinates commitment-based work assignment with external SNARK workers
- Manages P2P sharing of completed SNARK work across network peers
- Provides quality-controlled SNARK proofs for block production
- Implements competitive fee-based work selection and timeout management

## Architecture Overview

### Core Components

- **Distributed Pool**: `DistributedPool<JobState, SnarkJobId>` for indexed job
  management and P2P synchronization
- **Candidate System**: Multi-stage validation pipeline for incoming work from
  peers
- **Commitment System**: Time-bound work assignments with automatic timeout
  handling
- **Priority Management**: Order-based job prioritization with external worker
  coordination

### Job Lifecycle Management

```
JobsUpdate → Available → Committed → Completed/TimedOut
                      ↘ WorkReceived → Verified → Promoted to Pool
```

1. **JobsUpdate** - receives available jobs from scan state, manages job
   retention and ordering
2. **Available** - jobs ready for commitment by local or remote workers
3. **Committed** - jobs assigned to workers with timeout tracking
4. **Completed** - verified SNARK work ready for block production use

## Features

### Commitment-Based Work Assignment

- **Auto-commitment creation** - automatically assigns jobs to available
  external workers
- **Competitive commitments** - accepts better commitments (higher fees) from
  network peers
- **Timeout management** - removes stale commitments that fail to deliver work
- **Strategy support** - configurable sequential vs random job selection
  strategies

### Distributed Pool Synchronization

- **Indexed messaging** - uses DistributedPool indices for efficient P2P range
  queries
- **Best tip validation** - only shares work with peers on compatible blockchain
  state
- **Orphaned work handling** - reintegrates valuable SNARK work when jobs change
- **Range-based fetching** - enables efficient bulk synchronization with peers

### Quality Control and Competition

- **Fee-based selection** - prioritizes higher-fee work for same jobs
- **Work comparison** - implements SNARK quality assessment
- **Candidate validation** - multi-stage pipeline ensures only verified work
  enters pool
- **Inferior work removal** - automatically removes lower-quality work for same
  jobs

## Integration Points

### External SNARK Worker Coordination

- **Automatic work assignment** - dispatches highest priority jobs to available
  workers
- **Work cancellation** - cancels work when jobs become obsolete
- **Availability tracking** - coordinates with worker capacity management
- **Strategy implementation** - supports different work selection approaches

### P2P Network Integration

- **Work announcements** - broadcasts completed work to network peers
- **Commitment sharing** - announces work commitments to establish priority
- **Candidate fetching** - retrieves and validates work from peer announcements
- **Synchronization** - maintains pool consistency across network participants

### Blockchain Integration

- **Scan state coordination** - receives available jobs from blockchain state
  changes
- **Block production support** - provides verified SNARK work for block creation
- **Transition frontier sync** - ensures work sharing only with synchronized
  peers
- **Job prioritization** - maintains work order based on blockchain requirements

## State Management

### Job State Tracking

- **Multi-field state** - tracks time, commitment, completed work, and priority
  order
- **Duration estimation** - calculates expected completion time based on job
  complexity
- **Status monitoring** - provides comprehensive job lifecycle visibility
- **Resource reporting** - tracks pool size, candidate status, and consistency
  metrics

### Timeout and Cleanup

- **Periodic timeout checking** - regularly validates commitment freshness
- **Automatic cleanup** - removes expired commitments and completed jobs
- **Orphaned work recovery** - preserves valuable work across job updates
- **Candidate pruning** - removes obsolete candidate work based on pool state

## Technical Implementation

### Distributed Pool Implementation

Uses `DistributedPool` for:

- **Index-based synchronization** - enables efficient P2P range queries
- **Deterministic ordering** - maintains consistent job sequence across nodes
- **Update tracking** - supports incremental synchronization with peers
- **Message generation** - provides standardized commitment and work
  announcements

### Unified Reducer Implementation

- **Single reducer pattern** - handles both state updates and action dispatching
- **Substate delegation** - coordinates with candidate subsystem via compatible
  substates
- **Effect coordination** - minimal effectful actions for service integration
  only
- **Deterministic execution** - ensures reproducible state transitions for
  debugging

## Technical Debt

This component mostly follows new patterns but has minor issues:

- **Incomplete Migration**: Still has a minimal `snark_pool_effects.rs` file
  with one effectful action
- **Error Handling**: TODO comments indicate missing error propagation
  (`// TODO: log or propagate`)
- **Pattern Consistency**: The presence of effects file suggests incomplete
  adoption of new unified reducer pattern

The remaining cleanup involves:

1. Moving the single effectful action to follow the thin effects pattern
2. Implementing proper error handling and propagation
3. Removing the separate effects file once migration is complete

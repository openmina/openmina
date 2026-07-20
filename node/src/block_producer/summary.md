# Block Producer State Machine

Orchestrates the complete block production pipeline from VRF evaluation to proof
generation and network broadcast.

## Purpose

- **Block Production Pipeline**: Coordinates the multi-phase process of
  creating, proving, and broadcasting blocks
- **VRF Integration**: Uses VRF evaluator subcomponent to determine slot
  leadership eligibility
- **Transaction Selection**: Retrieves and includes transactions from the
  transaction pool based on fee priority
- **Proof Coordination**: Requests block proofs from external SNARK services and
  handles proof completion
- **Network Broadcasting**: Injects completed blocks into the P2P network via
  transition frontier

## Architecture

### State Structure

- **BlockProducerState**: Optional wrapper enabling/disabling block production
- **BlockProducerEnabled**: Active state containing configuration, VRF
  evaluator, current state, and injected block tracking
- **VRF Evaluator**: Subcomponent handling slot leadership determination (see
  `vrf_evaluator/summary.md`)
- **Injected Blocks Tracking**: `BTreeSet<StateHash>` maintaining blocks pending
  best tip transitions

### Multi-Phase Block Production State Flow

```
Idle → WonSlot → WonSlotWait → WonSlotProduceInit →
WonSlotTransactionsGet → WonSlotTransactionsSuccess →
StagedLedgerDiffCreatePending → StagedLedgerDiffCreateSuccess →
BlockUnprovenBuilt → BlockProvePending → BlockProveSuccess →
BlockProduced → BlockInjected → Idle
```

### Action Types

- **VRF Actions**: `VrfEvaluator(BlockProducerVrfEvaluatorAction)` for slot
  leadership
- **Timing Actions**: `WonSlotSearch`, `WonSlot`, `WonSlotWait` for slot
  coordination
- **Transaction Actions**: `WonSlotTransactionsGet/Success` for mempool
  integration
- **Ledger Actions**: `StagedLedgerDiffCreate*` for state transition
  construction
- **Proof Actions**: `BlockProve*` for external SNARK proof coordination
- **Network Actions**: `BlockInject/Injected` for P2P broadcast

## Key Algorithms

### Block Production Coordination

1. **VRF Evaluation**: Delegates to VRF evaluator subcomponent for slot wins
2. **Transaction Collection**: Requests transactions from pool sorted by fee
3. **Staged Ledger Diff**: Creates valid state transitions including coinbase
   and fees
4. **Block Construction**: Builds unproven blocks with all required components
5. **Proof Generation**: Coordinates with external services for block SNARK
   proofs
6. **Network Injection**: Broadcasts completed blocks via transition frontier

### Timing Management

- **Slot Boundaries**: Ensures production occurs within 3-minute slot windows
- **Best Tip Tracking**: Maintains won slots valid for current blockchain state
- **Production Delays**: 1-second broadcast delay for network time
  synchronization
- **Sync Awareness**: Pauses production during frontier synchronization

## Service Integration

### VRF Evaluator Subcomponent

- **Slot Leadership**: Determines eligibility based on stake and VRF evaluation
- **Epoch Management**: Handles epoch transitions and staking ledger switches
- **Won Slot Caching**: Provides future slot wins for production scheduling

### Ledger Service

- **Transaction Retrieval**: Gets pending transactions sorted by fee priority
- **Staged Ledger Diff**: Creates valid state transitions with transaction
  inclusion
- **State Validation**: Ensures proper coinbase, fee, and proof handling

### External Proof Services

- **Block Proving**: Requests SNARK proofs for complete block validation
- **Proof Integration**: Incorporates proofs into final block structure

### P2P Network

- **Block Broadcasting**: Injects blocks into gossip network via transition
  frontier
- **Network Timing**: Coordinates broadcast timing for consistent propagation

## Technical Debt

### Critical Issues

- **Block Proof Failure Handling**: `BlockProducerEvent::BlockProve` includes
  error handling via `Result<Arc<MinaBaseProofStableV2>, String>`, but error
  cases trigger `todo!()` panics in event source processing rather than proper
  error sink service integration for graceful failure handling
- **Currency Overflow Handling**: `todo!("total_currency overflowed")` in block
  production when currency calculations overflow, indicating incomplete edge
  case handling
- **Missing Error Recovery**: No fallback mechanisms for proof failures - system
  should integrate with error sink service instead of panicking

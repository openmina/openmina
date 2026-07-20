# Ledger State Machine

Manages the blockchain's account ledger, balances, and ledger synchronization
through coordinated read/write operations.

## Purpose

- Maintains account states and balances in snarked and staged ledgers
- Applies transactions and blocks to update ledger state
- Provides merkle proofs and account lookups for various consumers
- Manages ledger synchronization with isolated sync state
- Tracks mask lifecycle and prevents memory leaks

## Key Components

- **Read Substate**: Cost-limited concurrent ledger queries with request
  deduplication
- **Write Substate**: Sequential ledger operations (block apply, diff creation,
  reconstruction, commits)
- **LedgerManager**: Thread-based service that handles actual ledger operations
  asynchronously
- **Sync State**: Separate ledger storage for synchronization operations
- **Archive Support**: Additional data collection for archive nodes

## Service Architecture

- **LedgerManager**: Spawns dedicated "ledger-manager" thread with message
  passing interface
- **Request Types**: Unified request enum covering read/write operations,
  account management, and sync operations
- **Async/Sync Modes**: Supports both fire-and-forget calls and synchronous
  blocking calls
- **State Machine Integration**: Routes responses back through event system to
  update state machines

## Storage Architecture

- **Snarked Ledgers**: Finalized ledger states indexed by merkle root hash
  (includes disk-loaded ledgers)
- **Staged Ledgers**: Working ledgers with pending transactions, indexed by
  staged ledger hash
- **Sync Ledgers**: Temporary storage during ledger synchronization

## Interactions

- **Transition Frontier**: Block application and synchronization coordination
- **Block Producer**: Staged ledger diff creation for new blocks
- **RPC/P2P**: Account queries, ledger sync, and proof generation
- **VRF Evaluator**: Delegator table construction for consensus

## Technical Debt

The ledger implementation has several areas of technical debt:

### Code Organization

- **Large service files**: LedgerManager and LedgerService are complex and need
  simplification/reorganization
- **Documentation gaps**: Core components like LedgerCtx need better
  documentation of their responsibilities

### Integration and Testing Issues

- **Heavy coupling**: Deep integration with transition frontier, block producer,
  and P2P makes isolated testing difficult
- **Mask leak detection**: Unreliable during testing scenarios (alive_masks
  tracking)
- **Threading complexity**: Staged ledger reconstruction spawns additional
  threads with callback patterns
- **Ad-hoc threading**: Manual thread spawning for reconstruction instead of
  async patterns
- **Workaround patterns**: TODO comments about making services async to remove
  threading workarounds

### Error Handling and Reliability

- **Inconsistent error handling**: Mix of panics, unwraps, and proper error
  propagation throughout
- **Type safety gaps**: Request/response relationships "can't be expressed in
  the Rust type system"
- **Debugging infrastructure**: Specialized dump-to-file functions suggest
  frequent debugging needs
- **Hash mismatch handling**: Panics on staged ledger hash mismatches instead of
  graceful recovery
- **Silent failures**: Some operations like commit fail silently with TODO
  comments questioning this behavior

### Configuration Issues

- **Hardcoded constants**: Ledger depth tied to mainnet constants - will break
  if networks use different depths

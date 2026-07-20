# Transition Frontier Sync State Machine

Orchestrates the complete transition frontier synchronization process through
sequential ledger sync phases followed by block fetching, application, and
commitment.

## Purpose

- Synchronizes transition frontier to a new best tip through multi-phase process
- Downloads and reconstructs all required ledger states (staking, next epoch,
  root)
- Fetches missing blocks between current root and new best tip
- Applies fetched blocks sequentially to build new chain
- Commits synchronized state to become the new transition frontier

## Sequential Synchronization Phases

### Phase 1: Bootstrap (Ledger Synchronization)

Sequential ledger synchronization for consensus operation:

1. **Staking Ledger Sync** - synchronizes staking epoch ledger (snarked only)
2. **Next Epoch Ledger Sync** - synchronizes next epoch ledger if different
   (snarked only)
3. **Root Ledger Sync** - synchronizes ledger at transition frontier root
   (snarked + staged)

Each ledger sync delegates to the ledger sync coordinator which handles snarked
and staged components.

### Phase 2: Catchup (Block Synchronization)

```
BlocksPending → BlocksSuccess → CommitPending → CommitSuccess → Synced
```

- **BlocksPending**: Fetches missing blocks from peers and applies them
  sequentially
- **BlocksSuccess**: All blocks fetched and applied successfully
- **CommitPending**: Commits the synchronized chain to become new transition
  frontier
- **CommitSuccess**: Commitment completed successfully
- **Synced**: Synchronization complete, transition frontier updated

## Multi-Peer Block Fetching

- **Parallel fetching** - requests blocks from multiple peers simultaneously
- **Retry logic** - retries failed block fetches with different peers
- **Sequential application** - applies blocks in order even if fetched out of
  order
- **Error recovery** - handles block application errors gracefully

## Root Snarked Ledger Updates

Tracks snarked ledger transitions that occur during sync to enable proper ledger
reconstruction:

- Maps new snarked ledger hashes to parent ledger and staged ledger information
- Enables reconstruction of intermediate snarked ledger states
- Required when root block changes during synchronization process

## State Flow

```
Idle → Init → StakingLedgerPending → StakingLedgerSuccess
           → NextEpochLedgerPending → NextEpochLedgerSuccess
           → RootLedgerPending → RootLedgerSuccess
           → BlocksPending → BlocksSuccess
           → CommitPending → CommitSuccess → Synced
```

## Key Features

- **Three-phase sync strategy** - bootstrap (ledgers) → catchup (blocks) →
  commit
- **Multi-peer resilience** - fetches from multiple peers with error recovery
- **Sequential application** - maintains block order during application process
- **Protocol state collection** - gathers needed protocol states throughout sync
- **Root update tracking** - handles snarked ledger changes during sync
- **Sync phase identification** - distinguishes bootstrap vs catchup vs synced
  states

## Interactions

- Delegates ledger synchronization to ledger sync coordinator
- Fetches missing blocks via P2P RPC from multiple peers
- Applies blocks sequentially using transition frontier application logic
- Commits synchronized state to update the transition frontier
- Collects and manages protocol states needed for consensus validation
- Coordinates with block application service for heavy computation

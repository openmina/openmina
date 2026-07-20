# Transition Frontier Genesis State Machine

Manages genesis block creation, proving, and chain initialization for fresh
blockchain startup.

## Purpose

- Loads genesis configuration data from external sources
- Produces genesis block with proper protocol state structure
- Generates real proofs for genesis block (block producers only)
- Provides genesis blocks for transition frontier initialization
- Supports both block producer and non-block-producer node configurations

## Genesis Block Creation Flow

```
Idle → LedgerLoadPending → LedgerLoadSuccess → Produced → ProvePending → ProveSuccess
```

1. **LedgerLoadPending** - loads genesis configuration from genesis effectful
   service
2. **LedgerLoadSuccess** - genesis configuration loaded successfully with ledger
   data
3. **Produced** - genesis block structure created with protocol state and stake
   proof
4. **ProvePending** - generating real blockchain proof for genesis block (block
   producers only)
5. **ProveSuccess** - genesis block with real proof completed

## Dual Proof Support

- **Real proofs** - full blockchain proofs generated for block producers
- **Dummy proofs** - placeholder proofs for non-block-producer nodes
- **Flexible access** - `block_with_real_or_dummy_proof()` provides appropriate
  block type

## Genesis Block Structure

- **Protocol state creation** - constructs negative-one and genesis protocol
  states
- **Stake proof generation** - creates producer stake proof for consensus
  validation
- **Empty block body** - genesis blocks contain no transactions (empty staged
  ledger diff)
- **Chain proof setup** - establishes genesis block as root for future chain
  proofs

## Key Features

- **Service integration** - delegates heavy genesis computation to effectful
  service
- **Block producer awareness** - only generates real proofs when node is
  producing blocks
- **Memory efficiency** - provides lightweight dummy proofs for non-producing
  nodes
- **Protocol state management** - handles negative-one and genesis protocol
  state creation
- **Stake proof support** - generates appropriate stake proofs for consensus
  operation

## Integration Points

- **Genesis effectful service** - loads configuration and performs heavy genesis
  computation
- **Transition frontier** - provides genesis block for chain initialization
- **Block production** - supplies real genesis proofs when node becomes block
  producer
- **Consensus** - provides stake proofs and protocol states for consensus
  validation

## Configuration Support

- **Genesis ledger loading** - loads initial account distribution and
  configuration
- **Protocol state setup** - establishes initial consensus parameters
- **Epoch configuration** - sets up staking and next epoch ledger information
- **Chain initialization** - provides foundation for transition frontier
  bootstrap

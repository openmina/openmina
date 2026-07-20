# Watched Accounts State Machine

**Originally designed for when the web node was a light client without full node
capabilities - fully integrated into the state machine but currently has no
entry points to trigger its functionality.**

Tracks specific account public keys to monitor their transaction activity and
ledger state changes.

## Purpose

- Maintains a registry of accounts to monitor by public key
- Detects when watched accounts appear in block transactions
- Retrieves account state from ledger after transaction inclusion
- Provides transaction history and account state tracking per block

## Account State Tracking

- **Initial State**: Fetches account data from current best tip ledger
- **Block Updates**: Tracks relevant transactions in each new block
- **Ledger Queries**: Retrieves updated account state after block inclusion
- **State Transitions**: Idle → Pending → Success/Error for both initial and
  per-block queries

## Transaction Detection

- **Diff Analysis**: Scans staged ledger diffs for account references
- **Account Matching**: Matches transactions by public key in fee payer or
  receiver roles
- **Transaction Storage**: Stores relevant transactions ordered by nonce

## Data Structure

- **WatchedAccountsState**: Map of public keys to account monitoring state
- **WatchedAccountState**: Per-account initial state + block history queue
- **WatchedAccountBlockState**: Per-block transaction list + ledger account data
- **Transaction Filtering**: Extracts UserCommands affecting the watched account

## Interactions

- **Transition Frontier**: Monitors new blocks for relevant transactions
- **P2P Network**: Queries peers for account data (currently TODO/disabled)
- **Ledger System**: Retrieves account state from specific ledger hashes

## Current Status

- **Historical context**: Built for light client use case before the web node
  had full node capabilities
- **Fully integrated**: Complete state machine implementation with proper
  reducers
- **No entry points**: No RPC endpoints, CLI commands, or triggers to activate
  functionality
- **P2P queries disabled**: Ledger query logic marked TODO in reducer
- **Ready for activation**: Could be enabled by adding appropriate triggers
- **Limited scope**: Only handles UserCommand transactions, no ZkApp support

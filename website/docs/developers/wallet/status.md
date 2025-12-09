---
title: status
description: Check the status of a submitted transaction
sidebar_position: 5
---

# status

Check the status of a submitted transaction using its hash.

## Basic usage

```bash
mina wallet status --hash <TRANSACTION_HASH>
```

## Arguments

**Required:**

- `--hash <HASH>` - Transaction hash to check

**Optional:**

- `--node <URL>` - Node GraphQL endpoint (default: `http://localhost:3000`)
- `--check-mempool` - Force checking the mempool even if transaction is found in
  blockchain

## Examples

### Check transaction on local node

```bash
mina wallet status \
  --hash 5Ju4H4DTE1zkwrnLrQ8vb2sZR19b7eSMiAVbb4wQh4bfhh4aQNew
```

### Check transaction on remote node

```bash
mina wallet status \
  --hash 5Ju4H4DTE1zkwrnLrQ8vb2sZR19b7eSMiAVbb4wQh4bfhh4aQNew \
  --node https://devnet-plain-1.gcp.o1test.net
```

## Output

The status command will:

1. First attempt to query the blockchain for the transaction status
2. If not found in the blockchain, automatically check the mempool (pending
   transactions)
3. Display transaction details if found in the mempool

### Transaction found in mempool

```
Checking transaction status...
Transaction hash: 5Ju4H4DTE1zkwrnLrQ8vb2sZR19b7eSMiAVbb4wQh4bfhh4aQNew

Transaction not found in blockchain, checking mempool...

✓ Transaction found in mempool!

Transaction Details:
  Hash:   5Ju4H4DTE1zkwrnLrQ8vb2sZR19b7eSMiAVbb4wQh4bfhh4aQNew
  From:   B62qjtpVAMr7knjLxRLU887QgT7GPk3JYCg8NGdZsfMuaykAJ9C2Rem
  To:     B62qjtpVAMr7knjLxRLU887QgT7GPk3JYCg8NGdZsfMuaykAJ9C2Rem
  Amount: 1000000000 nanomina
  Fee:    1000000 nanomina
  Nonce:  0

Status: PENDING (waiting to be included in a block)
```

### Transaction not found

```
Checking transaction status...
Transaction hash: 5Ju6ku4DY5McpfqPvduQyQASjv1iAF12Xn75W3f3kGL1wsgSRKBA

Transaction not found in blockchain, checking mempool...

✗ Transaction not found in mempool

The transaction may have:
  - Already been included in a block
  - Been rejected by the network
  - Not yet propagated to this node
```

## How it works

The status command automatically:

1. **Queries blockchain** - Attempts to query `transactionStatus` via GraphQL
2. **Falls back to mempool** - If not found or if the query fails, checks
   `pooledUserCommands` for pending transactions
3. **Displays results** - Shows transaction details if found, or helpful
   messages if not found

This is particularly useful immediately after sending a transaction to verify it
has been accepted into the mempool.

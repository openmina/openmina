---
sidebar_position: 0
title: Mina as a database
description: Understanding Mina through traditional database concepts
---

# Mina: decentralized programmable and verifiable database

This guide explains Mina blockchain concepts using traditional database
terminology, making it accessible to developers familiar with databases but new
to web3.

## What is Mina?

Mina is the **first programmable decentralized verifiable key-value storage**.
Unlike traditional databases that require trusting an operator, Mina provides
cryptographic guarantees that the data is correct and hasn't been tampered with.

The key innovation is **succinct verification**: you can verify the entire
database state with a ~22KB proof, regardless of how much data is stored.

## Core concepts mapping

### Data storage layer

| Mina concept           | Database equivalent                        |
| ---------------------- | ------------------------------------------ |
| **Ledger**             | Key-value data store                       |
| **Account**            | Record/Row                                 |
| **Public key**         | Primary key                                |
| **Account state**      | Record value (balance, nonce, app state)   |
| **zkApp state fields** | Columns within a record (8 field elements) |
| **Merkle tree root**   | Database checksum/hash                     |

The ledger is fundamentally a key-value store where:

- **Key** = Public key (account address)
- **Value** = Account state (balance, nonce, permissions, zkApp state)

### Operations layer

| Mina concept    | Database equivalent                     |
| --------------- | --------------------------------------- |
| **Transaction** | Write operation / State mutation        |
| **Block**       | Batched transaction commit / WAL entry  |
| **Mempool**     | Pending writes queue / Operation buffer |
| **Nonce**       | Sequence number for ordering writes     |
| **Fee**         | Priority for write processing           |

Transactions are the write operations that modify the ledger state. They're
batched into blocks, similar to how databases batch writes for efficiency.

### Authentication layer

| Mina concept    | Database equivalent           |
| --------------- | ----------------------------- |
| **Signature**   | Authentication token          |
| **Private key** | Password / Secret credential  |
| **Public key**  | Username / Account identifier |

Every write operation must be authenticated with a digital signature, proving
the requester owns the account they're modifying.

### Authorization layer (zkApps)

| Mina concept         | Database equivalent                  |
| -------------------- | ------------------------------------ |
| **zkApp**            | Stored procedure with access control |
| **Permissions**      | Access control rules (GRANT/REVOKE)  |
| **Verification key** | Schema definition + access policy    |
| **Preconditions**    | Read-before-write assertions         |
| **ZK proof**         | Verified authorization predicate     |

zkApps are programmable accounts that define custom rules for state updates.
Think of them as stored procedures that enforce business logic and access
control at the database level.

**Preconditions** act like optimistic locking - they specify conditions that
must be true about the current state for a write to succeed.

### Logging and audit layer

| Mina concept     | Database equivalent                         |
| ---------------- | ------------------------------------------- |
| **Archive node** | Historical transparency log / Audit trail   |
| **Events**       | Real-time append-only log / CDC stream      |
| **Actions**      | Sequenced event queue for processing        |
| **Block hash**   | Commit ID / Transaction log sequence number |

**Archive nodes** store the complete history of all state changes, enabling
queries about past states and providing an audit trail.

**Events** are emitted by zkApps during execution, functioning like change data
capture (CDC) streams that external systems can subscribe to.

### Infrastructure layer

| Mina concept            | Database equivalent                          |
| ----------------------- | -------------------------------------------- |
| **Consensus**           | Distributed write ordering (Paxos/Raft-like) |
| **Node**                | Database replica                             |
| **Transition frontier** | Rolling checkpoint window                    |
| **Scan state**          | Proof aggregation pipeline                   |
| **SNARK workers**       | Background verification workers              |
| **Best tip**            | Current committed state                      |

The consensus protocol ensures all nodes agree on the order of writes, similar
to how distributed databases use consensus protocols like Paxos or Raft, but
permissionless.

## What makes Mina unique

### Succinct verification

Traditional databases require trusting the operator or auditing all data. Mina
produces a constant-size (~22KB) cryptographic proof that:

- The entire state history is valid
- All transactions followed the rules
- No data was tampered with

Any client can verify this proof in milliseconds, regardless of database size.

### Programmable authorization

Unlike traditional row-level security, zkApps can enforce arbitrary logic:

```
Traditional DB: "User X can write to rows where owner = X"
Mina zkApp:     "Account can be updated if the requester proves they solved
                 this specific computation correctly"
```

The proof is verified without revealing the inputs, enabling privacy-preserving
authorization rules.

### Decentralized operation

- No single operator controls the database
- Anyone can run a node and verify state
- Write ordering is determined by consensus, not a central authority
- Censorship resistance: valid transactions cannot be permanently blocked

## Practical examples

### Reading data

```
Traditional:  SELECT balance FROM accounts WHERE address = '0x...'
Mina:         Query any node for account state at a public key
              The response includes a proof of correctness
```

### Writing data

```
Traditional:  UPDATE accounts SET balance = balance - 10 WHERE address = '0x...'
Mina:         Create a signed transaction specifying the state change
              Submit to any node's mempool
              Wait for inclusion in a block (commit)
```

### Complex writes with zkApps

```
Traditional:  CALL stored_procedure('transfer', from, to, amount)
              (trusts DB to enforce procedure logic)

Mina:         Generate ZK proof that your inputs satisfy the zkApp's rules
              Submit transaction with proof attached
              Anyone can verify the rules were followed without seeing inputs
```

## Summary

| Traditional database | Mina                                        |
| -------------------- | ------------------------------------------- |
| Trust the operator   | Verify with cryptographic proofs            |
| Centralized          | Decentralized (thousands of nodes)          |
| SQL queries          | Account state queries with inclusion proofs |
| Stored procedures    | zkApps with ZK proof authorization          |
| Audit logs           | Archive nodes + events (tamper-proof)       |
| Access control       | Permissions + ZK proofs                     |

Mina brings database concepts into a trustless environment where verification
replaces trust, and cryptographic proofs replace administrative controls.

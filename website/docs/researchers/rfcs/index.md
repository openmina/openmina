---
sidebar_position: 1
title: Mina Protocol RFCs
description: Request for Comments documents for the Mina Protocol
slug: /researchers/rfcs
---

# Mina Protocol RFCs

This section contains all Request for Comments (RFC) documents from the
[Mina Protocol OCaml implementation](https://github.com/MinaProtocol/mina).
These RFCs document design decisions, protocol changes, and architectural
proposals for the Mina blockchain.

RFCs serve as the primary mechanism for proposing new features, collecting
community input, and documenting design decisions. They provide valuable context
for understanding why certain architectural choices were made in the protocol.

The original RFCs are maintained in the
[MinaProtocol/mina repository](https://github.com/MinaProtocol/mina/tree/compatible/rfcs).

## RFC categories

### Protocol and consensus

Core protocol design, consensus mechanisms, and blockchain state management.

| RFC                                     | Title                 | Description                                   |
| --------------------------------------- | --------------------- | --------------------------------------------- |
| [0006](./0006-receipt-chain-proving.md) | Receipt chain proving | Proof mechanism for transaction receipts      |
| [0007](./0007-delegation-of-stake.md)   | Delegation of stake   | Stake delegation mechanics and implementation |
| [0019](./0019-epoch-ledger-sync.md)     | Epoch ledger sync     | Synchronization of epoch ledgers              |
| [0030](./0030-fork-signalling.md)       | Fork signalling       | Mechanism for signaling protocol forks        |
| [0051](./0051-protocol-versioning.md)   | Protocol versioning   | Versioning scheme for protocol compatibility  |
| [0059](./0059-new-transaction-model.md) | New transaction model | Redesigned transaction model                  |

### State management

Transition frontier, ledger, and state persistence.

| RFC                                                    | Title                                | Description                               |
| ------------------------------------------------------ | ------------------------------------ | ----------------------------------------- |
| [0008](./0008-persistent-ledger-builder-controller.md) | Persistent ledger builder controller | Controller for persistent ledger building |
| [0009](./0009-transition-frontier-controller.md)       | Transition frontier controller       | Managing the transition frontier          |
| [0010](./0010-decompose-ledger-builder.md)             | Decompose ledger builder             | Modular ledger builder design             |
| [0015](./0015-transition-frontier-extensions.md)       | Transition frontier extensions       | Extensions to transition frontier         |
| [0016](./0016-transition-frontier-persistence.md)      | Transition frontier persistence      | Persisting frontier state                 |
| [0020](./0020-transition-frontier-extensions-2.md)     | Transition frontier extensions 2     | Additional frontier extensions            |
| [0026](./0026-transition-caching.md)                   | Transition caching                   | Caching strategies for transitions        |
| [0028](./0028-frontier-synchronization.md)             | Frontier synchronization             | Synchronizing frontiers across nodes      |
| [0034](./0034-reduce-scan-state-memory-usage.md)       | Reduce scan state memory usage       | Memory optimization for scan state        |

### Networking

P2P communication, libp2p integration, and network architecture.

| RFC                                   | Title               | Description                       |
| ------------------------------------- | ------------------- | --------------------------------- |
| [0029](./0029-libp2p.md)              | libp2p              | libp2p integration for networking |
| [0031](./0031-sentry-architecture.md) | Sentry architecture | Sentry node architecture design   |
| [0060](./0060-networking-refactor.md) | Networking refactor | Overhauling the networking layer  |
| [0062](./0062-bitswap.md)             | Bitswap             | Bitswap protocol integration      |

### APIs and interfaces

GraphQL, RPC, Rosetta, and external interfaces.

| RFC                                        | Title                    | Description                          |
| ------------------------------------------ | ------------------------ | ------------------------------------ |
| [0013](./0013-rpc-versioning.md)           | RPC versioning           | Versioning scheme for RPC interfaces |
| [0021](./0021-graphql-api.md)              | GraphQL API              | GraphQL API for wallet communication |
| [0038](./0038-rosetta-construction-api.md) | Rosetta Construction API | Rosetta API construction endpoints   |
| [0040](./0040-rosetta-timelocking.md)      | Rosetta timelocking      | Timelocking support in Rosetta       |
| [0048](./0048-rosetta-zkapps.md)           | Rosetta zkApps           | zkApps support in Rosetta API        |

### Hard forks

Hard fork procedures, disaster recovery, and data migration.

| RFC                                                | Title                            | Description                        |
| -------------------------------------------------- | -------------------------------- | ---------------------------------- |
| [0033](./0033-blockchain-in-hard-fork.md)          | Blockchain in hard fork          | Blockchain state during hard forks |
| [0035](./0035-scan-state-hard-fork.md)             | Scan state hard fork             | Scan state handling in hard forks  |
| [0036](./0036-hard-fork-disaster-recovery.md)      | Hard fork disaster recovery      | Recovery procedures for hard forks |
| [0047](./0047-versioning-changes-for-hard-fork.md) | Versioning changes for hard fork | Version management during forks    |
| [0053](./0053-hard-fork-package-generation.md)     | Hard fork package generation     | Generating hard fork packages      |
| [0056](./0056-hard-fork-data-migration.md)         | Hard fork data migration         | Data migration during hard forks   |

### zkApps

Zero-knowledge application features and constraints.

| RFC                                             | Title                          | Description                             |
| ----------------------------------------------- | ------------------------------ | --------------------------------------- |
| [0045](./0045-zkapp-balance-data-in-archive.md) | zkApp balance data in archive  | Archive storage for zkApp balances      |
| [0052](./0052-verification-key-permissions.md)  | Verification key permissions   | Permission system for verification keys |
| [0054](./0054-limit-zkapp-cmds-per-block.md)    | Limit zkApp commands per block | Block-level zkApp command limits        |
| [0057](./0057-hardcap-zkapp-commands.md)        | Hardcap zkApp commands         | Hard limits on zkApp commands           |
| [0058](./0058-disable-zkapp-commands.md)        | Disable zkApp commands         | Mechanism to disable zkApp commands     |
| [0061](./0061-solidity-snapps.md)               | Solidity SNAPPs                | Solidity integration for SNAPPs         |
| [0064](./0064-deriving-with-generics-snapps.md) | Deriving with generics SNAPPs  | Generic derivation for SNAPPs           |

### Security and validation

Transaction pool security, ban scoring, and validation mechanisms.

| RFC                                           | Title                           | Description                          |
| --------------------------------------------- | ------------------------------- | ------------------------------------ |
| [0001](./0001-banlisting.md)                  | Banlisting                      | Peer banlisting mechanism            |
| [0011](./0011-txpool-dos-mitigation.md)       | Transaction pool DoS mitigation | Preventing DoS attacks on mempool    |
| [0012](./0012-ban-scoring.md)                 | Ban scoring                     | Scoring system for peer bans         |
| [0032](./0032-automated-validation.md)        | Automated validation            | Automated transaction validation     |
| [0049](./0049-protocol-testing.md)            | Protocol testing                | Testing framework for protocol       |
| [0055](./0055-stop-transaction-processing.md) | Stop transaction processing     | Emergency transaction halt mechanism |

### Serialization and encoding

Data encoding, versioning, and serialization formats.

| RFC                                            | Title                        | Description                          |
| ---------------------------------------------- | ---------------------------- | ------------------------------------ |
| [0014](./0014-address-encoding.md)             | Address encoding             | Address encoding format              |
| [0017](./0017-module-versioning.md)            | Module versioning            | Versioning for serialization modules |
| [0024](./0024-memos-with-arbitrary-bytes.md)   | Memos with arbitrary bytes   | Arbitrary byte support in memos      |
| [0046](./0046-version-other-serializations.md) | Version other serializations | Versioning additional serializations |

### Account features

Time-locked accounts, delegations, and account management.

| RFC                                     | Title                 | Description                   |
| --------------------------------------- | --------------------- | ----------------------------- |
| [0025](./0025-time-locked-accounts.md)  | Time-locked accounts  | Time-based account locking    |
| [0050](./0050-genesis-ledger-export.md) | Genesis ledger export | Exporting genesis ledger data |

### Infrastructure and operations

Node status, logging, and operational tooling.

| RFC                                                  | Title                              | Description                         |
| ---------------------------------------------------- | ---------------------------------- | ----------------------------------- |
| [0018](./0018-better-logging.md)                     | Better logging                     | Improved logging infrastructure     |
| [0039](./0039-snark-keys-management.md)              | SNARK keys management              | Managing SNARK proving keys         |
| [0041](./0041-infra-testnet-persistence.md)          | Infrastructure testnet persistence | Persistent testnet infrastructure   |
| [0042](./0042-node-status-collection.md)             | Node status collection             | Collecting node status data         |
| [0043](./0043-node-error-collection.md)              | Node error collection              | Collecting node error data          |
| [0044](./0044-node-status-and-node-error-backend.md) | Node status and error backend      | Backend for status/error collection |
| [0063](./0063-reducing-daemon-memory-usage.md)       | Reducing daemon memory usage       | Memory optimization for daemon      |

### Development processes

Style guides, naming conventions, and development workflows.

| RFC                                           | Title                       | Description                     |
| --------------------------------------------- | --------------------------- | ------------------------------- |
| [0000](./0000-template.md)                    | Template                    | RFC template for new proposals  |
| [0002](./0002-branch-prefixes.md)             | Branch prefixes             | Git branch naming conventions   |
| [0003](./0003-renaming-refactor.md)           | Renaming refactor           | Code renaming guidelines        |
| [0004](./0004-style-guidelines.md)            | Style guidelines            | Code style guidelines           |
| [0005](./0005-issue-labels.md)                | Issue labels                | GitHub issue labeling scheme    |
| [0022](./0022-postake-naming-conventions.md)  | PoStake naming conventions  | Naming conventions for PoS code |
| [0023](./0023-glossary-terms.md)              | Glossary terms              | Protocol terminology glossary   |
| [0027](./0027-wallet-internationalization.md) | Wallet internationalization | i18n support for wallet         |
| [0037](./0037-github-merging-strategy.md)     | GitHub merging strategy     | Git merge workflow              |

## Additional resources

- [RFC Repository](https://github.com/MinaProtocol/mina/tree/compatible/rfcs) -
  Original RFC source files on GitHub
- [Mina Protocol Documentation](https://docs.minaprotocol.com/) - Official
  protocol documentation

---
sidebar_position: 1
title: Node architecture
description: Node startup, runtime services, and code navigation guide
slug: /developers/architecture
---

<!-- prettier-ignore-start -->

:::info Work in progress

This page is actively being written and will be updated over time as we document
the codebase. For comprehensive API details, refer to the
[Rust API documentation](https://o1-labs.github.io/mina-rust/api-docs/).

:::

<!-- prettier-ignore-end -->

# Node architecture

This page describes what happens when the node starts and where to find key
functionality in the codebase.

## Node startup

When you run `mina node`, the
[`Node::run`](https://o1-labs.github.io/mina-rust/api-docs/mina_cli/commands/node/struct.Node.html#method.run)
method initializes logging, configuration, P2P identity, SNARK verifiers (see
[Verifier](#verifier)), optional services, and starts the event loop.

## Runtime services

Services run in separate threads and communicate with the state machine via
channels. The main service container is
[`NodeService`](https://o1-labs.github.io/mina-rust/api-docs/mina_node_common/service/struct.NodeService.html).

| Service               | Purpose                                | Module                                    |
| --------------------- | -------------------------------------- | ----------------------------------------- |
| Archive               | Stores blockchain history              | `node/common/src/service/archive/`        |
| Block producer        | VRF evaluation and block proving       | `node/common/src/service/block_producer/` |
| Ledger manager        | Manages ledger database operations     | `node/src/ledger/`                        |
| P2P                   | Network connectivity (libp2p + WebRTC) | `p2p/`                                    |
| RPC                   | External API (GraphQL, JSON-RPC)       | `node/common/src/service/rpc/`            |
| SNARK worker          | Generates transaction proofs           | `node/common/src/service/snark_worker.rs` |
| [Verifier](#verifier) | Verifies SNARK proofs                  | `node/common/src/service/snarks.rs`       |

### Verifier

The verifier runs in a dedicated thread and validates SNARK proofs using
[Kimchi](https://github.com/o1-labs/proof-systems). At startup, it generates
verification keys from circuit definitions; these keys are required by the
proving system to verify proofs:
[`BlockVerifier`](https://o1-labs.github.io/mina-rust/api-docs/mina_tree/proofs/verifiers/struct.BlockVerifier.html)
for consensus-layer block proofs and
[`TransactionVerifier`](https://o1-labs.github.io/mina-rust/api-docs/mina_tree/proofs/verifiers/struct.TransactionVerifier.html)
for transaction and zkApp proofs.

The verifier handles three types of proofs:

- **Block proofs** - validates that a block was produced correctly according to
  consensus rules
  ([`verify_block()`](https://o1-labs.github.io/mina-rust/api-docs/mina_tree/proofs/verification/fn.verify_block.html))
- **Transaction proofs** - validates payment and delegation transactions
  ([`verify_transaction()`](https://o1-labs.github.io/mina-rust/api-docs/mina_tree/proofs/verification/fn.verify_transaction.html))
- **zkApp proofs** - validates smart contract executions
  ([`verify_zkapp()`](https://o1-labs.github.io/mina-rust/api-docs/mina_tree/proofs/verification/fn.verify_zkapp.html))

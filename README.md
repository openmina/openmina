<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/OpenMinaGH_Light.svg">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/OpenMinaGH_Dark.svg">
    <img alt="The Open Mina Node is a fast and secure implementation of the Mina protocol in Rust."
         src="docs/assets/OpenMinaGH_Light.svg"
         width="152px">
  </picture>

![Beta][beta-badge] [![release-badge]][release-link] [![Changelog][changelog-badge]][changelog] [![Apache licensed]][Apache link]

_The **Open Mina Node** is a fast and secure implementation of the Mina protocol in **Rust**._  
_Currently in **public beta**, join our [Discord community](https://discord.com/channels/484437221055922177/1290662938734231552) to help test future releases._

</div>

---

## Getting Started

### Building from Source

- [Rust Node](/docs/building-from-source-guide.md#how-to-build-and-launch-a-node-from-source) and [Dashboards](./docs/building-from-source-guide.md#how-to-launch-the-ui)

### Run Node on Devnet via Docker

- [Non-Block Producing Node](/docs/alpha-testing-guide.md) Connect to peers and sync a node on the devnet; no devnet stake needed.
- [Block Producing Node](/docs/block-producer-guide.md) Produce blocks on the devnet; sufficient devnet stake needed.
- [Local Block Production Demo](/docs/local-demo-guide.md) Produce blocks on a custom local chain without devnet stake.

<img src="docs/assets/NodeUI.png" alt="Block production Node UI">

---

## Release Process

**This project is in beta**. We maintain a monthly release cycle, providing [updates every month](https://github.com/openmina/openmina/releases).  



## Core Features

- **Mina Network**: Connect to peers, sync up, broadcast messages
- **Block Production**: Produces, validates, and applies blocks according to Mina's consensus.
- **SNARK Generation**: Produce SNARK proofs for transactions
- **Debugging**: A block replayer that uses data from the archive nodes

## Repository Structure

- [core/](core) - Provides basic types needed to be shared across different components of the node.
- [ledger/](ledger) - Mina ledger implementation in Rust.
- [snark/](snark) - Snark/Proof verification.
- [p2p/](p2p) - P2p implementation for OpenMina node.
- [node/](node) - Combines all the business logic of the node.
  - [native/](node/native) - OS specific pieces of the node, which is used to run the node natively (Linux/Mac/Windows).
  - [testing/](node/testing) - Testing framework for OpenMina node.
- [cli/](cli) - OpenMina cli.
- [frontend/](frontend) - OpenMina frontend.

## The Open Mina Documentation

### What is Open Mina?

- [Why we are developing Open Mina](docs/why-openmina.md)

### Core components

- [P2P communication](https://github.com/openmina/openmina/blob/documentation/docs/p2p_service.md)
  - [GossipSub](https://github.com/openmina/mina-wiki/blob/3ea9041e52fb2e606918f6c60bd3a32b8652f016/p2p/mina-gossip.md)
- [Scan state](docs/scan-state.md)
- [SNARKs](docs/snark-work.md)

### Developer tools

- [Front End](./docs/building-from-source-guide.md#how-to-launch-the-ui)

### Testing Framework for Mina

- [Full Testing Documentation](docs/testing/testing.md)

### How to run

- [Non-Block Producing Node](./docs/alpha-testing-guide.md)
- [Block Producing Node](./docs/block-producer-guide.md)
- [Local Block Production Demo](./docs/local-demo-guide.md)

[changelog]: ./CHANGELOG.md
[beta-badge]: https://img.shields.io/badge/status-beta-yellow
[changelog-badge]: https://img.shields.io/badge/changelog-Changelog-%23E05735
[release-badge]: https://img.shields.io/github/v/release/openmina/openmina
[release-link]: https://github.com/openmina/openmina/releases/latest
[Apache licensed]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[Apache link]: https://github.com/openmina/openmina/blob/master/LICENSE

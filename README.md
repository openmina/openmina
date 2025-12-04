<div align="center">
  <img src="website/static/img/rust-node-social-card.svg" alt="Mina Rust Node - Fast and secure implementation of the Mina protocol in Rust" width="600px">

![Beta][beta-badge] [![release-badge]][release-link]
![GitHub Repo stars](https://img.shields.io/github/stars/o1-labs/mina-rust?style=social)
[![Changelog][changelog-badge]][changelog] [![Apache licensed]][Apache link]

_The **Mina Rust Node** is a fast and secure implementation of the Mina protocol
in **Rust**._ _Currently in **public beta**, join our
[Discord community](https://discord.com/channels/484437221055922177/1290662938734231552)
to help test future releases._

</div>

---

## What is Mina Protocol?

Mina is a lightweight blockchain using zero-knowledge proofs to maintain a
constant blockchain size, making it the world's lightest blockchain. Learn more
at **[minaprotocol.com](https://minaprotocol.com)**.

## Quick Start

**[View full system requirements and setup instructions →](https://o1-labs.github.io/mina-rust/docs/node-operators/getting-started)**

## Getting Started

For comprehensive installation and setup instructions, visit our documentation
website:

**[Complete Setup Guide →](https://o1-labs.github.io/mina-rust/docs/node-operators/getting-started)**

### Quick Links

- **[Docker Installation](https://o1-labs.github.io/mina-rust/docs/node-operators/docker-usage)**
- **[Building from Source](https://o1-labs.github.io/mina-rust/docs/node-operators/building-from-source)**
- **[Block Producer Setup](https://o1-labs.github.io/mina-rust/docs/node-operators/block-producer)**
- **[Archive Node](https://o1-labs.github.io/mina-rust/docs/node-operators/archive-node)**

<img src="website/static/img/NodeUI.png" alt="Block production Node UI">

---

## Release Process

**This project is in beta**. We maintain a monthly release cycle, providing
[updates every month](https://github.com/o1-labs/mina-rust/releases).

## Core Features

The Mina Rust Node implements the complete Mina protocol in Rust, including
network connectivity, block production, SNARK generation, and debugging tools.

**[Learn More About Architecture →](https://o1-labs.github.io/mina-rust/docs/developers/getting-started)**

## Repository Structure

This repository contains the complete Mina Rust Node implementation:

- [core/](core) - Provides basic types needed to be shared across different
  components of the node.
- [ledger/](ledger) - Mina ledger implementation in Rust.
- [snark/](snark) - Snark/Proof verification.
- [p2p/](p2p) - P2p implementation for Mina node.
- [node/](node) - Combines all the business logic of the node.
  - [native/](node/native) - OS specific pieces of the node, which is used to
    run the node natively (Linux/Mac/Windows).
  - [testing/](tools/testing) - Testing framework for Mina node.
- [cli/](cli) - Mina CLI.
- [frontend/](frontend) - OpenMina frontend.

**[Learn more about the architecture →](https://o1-labs.github.io/mina-rust/docs/developers/getting-started)**

## Community & Support

**[Visit our comprehensive documentation website →](https://o1-labs.github.io/mina-rust)**

### Get Help & Contribute

- **[GitHub Discussions](https://github.com/o1-labs/mina-rust/discussions)** -
  Ask questions and share ideas
- **[Issues](https://github.com/o1-labs/mina-rust/issues)** - Report bugs or
  request features
- **[Discord Community](https://discord.com/channels/484437221055922177/1290662938734231552)** -
  Real-time support and testing
- **[Contributing Guide](https://o1-labs.github.io/mina-rust/docs/developers/getting-started)** -
  How to contribute code

### Key Documentation Sections

- **[Node Operators](https://o1-labs.github.io/mina-rust/docs/node-operators/getting-started)** -
  Installation and operation guides
- **[Developers](https://o1-labs.github.io/mina-rust/docs/developers/getting-started)** -
  Architecture and contribution guides
- **[API Documentation](https://o1-labs.github.io/mina-rust/api-docs/)** -
  Comprehensive API reference

## Supported Platforms

[![CI Status][ci-badge]][ci-link]

| Platform                                | Architecture  | Build Status                                                     |
| --------------------------------------- | ------------- | ---------------------------------------------------------------- |
| ![Ubuntu][ubuntu-icon] **Ubuntu 22.04** | x64           | [![Ubuntu 22.04 x64][ubuntu-22-badge]][ubuntu-22-link]           |
| ![Ubuntu][ubuntu-icon] **Ubuntu 24.04** | x64           | [![Ubuntu 24.04 x64][ubuntu-24-badge]][ubuntu-24-link]           |
| ![Ubuntu][ubuntu-icon] **Ubuntu 24.04** | ARM64         | [![Ubuntu 24.04 ARM64][ubuntu-24-arm-badge]][ubuntu-24-arm-link] |
| ![macOS][macos-icon] **macOS 13**       | Intel         | [![macOS 13 Intel][macos-13-badge]][macos-13-link]               |
| ![macOS][macos-icon] **macOS 14**       | Apple Silicon | [![macOS 14 M1/M2][macos-14-badge]][macos-14-link]               |
| ![macOS][macos-icon] **macOS 15**       | Apple Silicon | [![macOS 15 M1/M2/M3][macos-15-badge]][macos-15-link]            |
| ![macOS][macos-icon] **macOS Latest**   | Apple Silicon | [![macOS Latest][macos-latest-badge]][macos-latest-link]         |

> **Note**: Multi-platform builds run automatically on `develop` and `main`
> branches. Pull requests run fast Ubuntu-only builds for quick feedback.

## Nightly Status

[![Documentation Scripts][doc-scripts-badge]][doc-scripts-link]
[![GraphQL API Tests][graphql-api-badge]][graphql-api-link]
[![Infrastructure Tests][infra-tests-badge]][infra-tests-link]
[![Remote GraphQL][remote-graphql-badge]][remote-graphql-link]
[![Board Carryover][board-carryover-badge]][board-carryover-link]

[changelog]: ./CHANGELOG.md
[beta-badge]: https://img.shields.io/badge/status-beta-yellow
[changelog-badge]: https://img.shields.io/badge/changelog-Changelog-%23E05735
[release-badge]: https://img.shields.io/github/v/release/o1-labs/mina-rust
[release-link]: https://github.com/o1-labs/mina-rust/releases/latest
[Apache licensed]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[Apache link]: https://github.com/o1-labs/mina-rust/blob/master/LICENSE

<!-- Platform support badges -->

[ci-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/tests.yaml/badge.svg?branch=develop
[ci-link]: https://github.com/o1-labs/mina-rust/actions/workflows/tests.yaml
[ubuntu-icon]:
  https://img.shields.io/badge/-Ubuntu-E95420?style=flat&logo=ubuntu&logoColor=white
[macos-icon]:
  https://img.shields.io/badge/-macOS-000000?style=flat&logo=apple&logoColor=white

<!-- Individual platform badges -->

[ubuntu-22-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-22-04.yaml/badge.svg?branch=develop
[ubuntu-24-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-24-04.yaml/badge.svg?branch=develop
[ubuntu-24-arm-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-24-04-arm.yaml/badge.svg?branch=develop
[macos-13-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-13.yaml/badge.svg?branch=develop
[macos-14-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-14.yaml/badge.svg?branch=develop
[macos-15-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-15.yaml/badge.svg?branch=develop
[macos-latest-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-latest.yaml/badge.svg?branch=develop

<!-- Platform-specific build links -->

[ubuntu-22-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-22-04.yaml
[ubuntu-24-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-24-04.yaml
[ubuntu-24-arm-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-ubuntu-24-04-arm.yaml
[macos-13-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-13.yaml
[macos-14-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-14.yaml
[macos-15-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-15.yaml
[macos-latest-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/build-macos-latest.yaml

<!-- Nightly workflow badges -->

[doc-scripts-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-scripts.yaml/badge.svg?branch=develop
[doc-scripts-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-scripts.yaml
[graphql-api-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-graphql-api.yaml/badge.svg?branch=develop
[graphql-api-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-graphql-api.yaml
[infra-tests-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-infrastructure.yaml/badge.svg?branch=develop
[infra-tests-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-docs-infrastructure.yaml
[remote-graphql-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-graphql-remote.yml/badge.svg?branch=develop
[remote-graphql-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/test-graphql-remote.yml
[board-carryover-badge]:
  https://github.com/o1-labs/mina-rust/actions/workflows/board-carryover-interation.yaml/badge.svg?branch=develop
[board-carryover-link]:
  https://github.com/o1-labs/mina-rust/actions/workflows/board-carryover-interation.yaml

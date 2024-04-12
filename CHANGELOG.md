# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fix incorrect condition when deciding which snarked ledgers need to be synchronize during bootstrap which would result in a failure when bootstrapping the node during the second epoch.
- Correctly handle the situation in which the best tip changes during staged ledger reconstruction, causing the reconstruct to produce a stale result.
- Fixed various edge cases in p2p layer (error propagation, disconnection, self-connection).

## [0.3.1] - 2024-04-05

### Changed

- Internal improvements to the actions logging mechanism.

### Fixed

- Corrected sync stats for accounts fetching during ledger sync.
- Pruning of kademlia streams and requests.

### Added

- Docker images tagged for each new release.
- Bootstrap process testing on CI.

## [0.3.0] - 2024-03-29

### Changed

- **Rust Toolchain**: Updated the minimum required Rust toolchain to version 1.77.
- **Networking**:
  - **Libp2p Replacement**: Transitioned from libp2p to a custom internal networking implementation. The transition will be finalized in the next release, completely removing the libp2p dependency.
    - **Gossipsub**: Pending support. Current version of the node performs initial bootstrapping but cannot stay synchronized with network broadcasts.
    - **Kademlia**: Partial implementation includes bootstrapping and FIND_NODE server/client functionalities.
    - **Identify Protocol**: Absent in this release, rendering the node unusable as a seed node.
- **Frontend**:
  - **Mobile Compatibility**: Enhanced support for mobile platforms, improving user experience across various devices.

### Fixed

- **Staged Ledger**: Resolved an issue where the ledger reconstruct step would block the state machine.
- **Node Communication**: Fixed a bug where nodes did not respond to ledger queries from bootstrapping peers, enhancing network cooperation.
- **Frontend**:
  - **Test Stability**: Addressed and fixed previously failing tests.
- **Backend**:
  - **HTTP RPC**: Corrected an error triggered when querying the `/state` endpoint.

### Added

- **Bootstrap Efficiency**:
  - **Ledger Synchronization**: Optimized the snarked ledger synchronization process during bootstrap, significantly reducing the time required.
  - **Genesis Ledger Loading**: Enhanced the loading mechanism for the genesis ledger, achieving much faster startup times.
- **Frontend Enhancements**:
  - Network Node DHT view.
  - Network Bootstrap Stats for real-time monitoring of network bootstrap statistics.
  - Main Dashboard view.
- **Backend Improvements**:
  - **JsonPath Support**: Enhanced the `/state` HTTP RPC endpoint with JsonPath support, offering more flexible state querying capabilities.

## [0.2.0] - 2024-02-29

### Changed

- Default Rust toolchain switched to stable channel (as of 1.75).
- Internal refactoring to how leaf actions in the state machine are organized.

### Fixed

- Node can now connect to the current berkeleynet after updates to:
  - Wire type definitions.
  - Verification, proving and circuits.
  - Ledger and transaction application logic.

### Added

- Ledger tests on CI.

## [0.1.0] - 2024-02-02

### Fixed

- Optimized scan state to reduce memory usage by avoiding duplication of data.
- Updated proof verification to be compatible with the current berkeleynet (rampup4).

### Added

- Introduced support for producing proofs (blocks, payments, zkApp transactions, and merge proofs) compatible with the current berkeleynet (rampup4), with pending support for generating circuits.
- Added the ability to produce blocks (excluding user transactions) for testing purposes.
- Implemented pruning of inferior blocks post synchronization with the best tip.
- Enhanced the testing framework and debugging support as follows:
  - Improved compatibility with the OCaml node within the testing framework.
  - Introduced declarations for essential invariants to be verified by the test framework, simulator, and replayer.
  - Integrated SNARK worker into the simulator for SNARK production.
  - Added functionality to store dumps of blocks and staged ledgers for inspection upon block application failure due to mismatches.

## [0.0.1] - 2023-12-22

First public release.

### Added

- Alpha version of the node which can connect and syncup to the berkeleynet network, and keep applying new blocks to maintain consensus state and ledger up to date.
- Web-based frontend for the node.

[Unreleased]: https://github.com/openmina/openmina/compare/v0.3.1...develop
[0.3.1]: https://github.com/openmina/openmina/releases/tag/v0.3.0...v0.3.1
[0.3.0]: https://github.com/openmina/openmina/releases/tag/v0.2.0...v0.3.0
[0.2.0]: https://github.com/openmina/openmina/releases/tag/v0.1.0...v0.2.0
[0.1.0]: https://github.com/openmina/openmina/releases/tag/v0.0.1...v0.1.0
[0.0.1]: https://github.com/openmina/openmina/releases/tag/v0.0.1

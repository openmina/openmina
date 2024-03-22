# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Change minimum Rust toolchain to 1.77.

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

[unreleased]: https://github.com/openmina/openmina/compare/v0.2.0...develop
[0.2.0]: https://github.com/openmina/openmina/releases/tag/v0.1.0...v0.2.0
[0.1.0]: https://github.com/openmina/openmina/releases/tag/v0.0.1...v0.1.0
[0.0.1]: https://github.com/openmina/openmina/releases/tag/v0.0.1

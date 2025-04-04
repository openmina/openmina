# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.16.0] - 2025-04-04

### Added

- **GraphQL**: More queries (snark pool, pending snark work, genesis block, ledger status).

### Changed

- **GraphQL**: Added more fields to `daemonStatus` query˙

### Fixed

- **GraphQL**: Some issues with accounts.
- **Block Producer**: Corner case that caused the won slot search to sometimes be interrupted at epoch bounds.

## [0.15.0] - 2025-03-13

### Added

- Restored support for snark workers.
- **Archive**: Support for storing blocks to AWS, GCP and filesystem.
- **Tooling**: WebRTC traffic sniffer.
- **GraphQL**:
  - `sendPayment` mutation.
  - `sendDelegation` mutation.
  - `pooledUserCommands` query.
  - `pooledZkappCommands` query.
  - Various other partially implemented queries expanded to ensure compatibility with the OCaml node.


### Changed

- **P2P**: Wait until full validation is complete before broadcasting transactions and completed works.
- **Transition frontier**: Perform cheap consensus operation first, and then the more expensive proof verification.
- **Transaction pool**: Unified libp2p and webrtc logic for the initial phase of handling transactions received from gossip network. As a result, processing of transactions received during bootstrap is delayed until the initial sync is complete.
- **Transaction pool**: Suspend processing during block production.

### Fixed

- **Transition frontier**: Rare race condition in the case of forks during block production that could result in dropping staged ledgers too early.
- **Webnode**: Replaced tokio channels which had a race condition that could crash the thread on WASM.
- **Transaction pool**: Verify zkApp proofs in a dedicated thread to avoid blocking the state machine.

## [0.14.0] - 2025-01-31

### Changed

- **Rust Toolchain**: Updated the minimum required Rust toolchain to version 1.84.
- **Proofs**: Optimizations (MSM, field inversion) that speed up proof production.

### Fixed

- **P2P**: Correct handling of yamux windows limits
- **P2P**: Wait until full validation is complete before broadcasting blocks.
- **WebRTC/P2P**: Handle propagation of messages received from the WebRTC network to libp2p's gossip network (blocks, snarks and transactions).
- **Transaction pool**: Fixed checks for deep account updates when pre-validating transactions.

### Added

- **Archive mode**: Added support for archive mode, which allows the node to connect to an archiver process and store node data in a database.

## [0.13.0] - 2025-01-06

### Fixed

- **Mempool**: Inside the transaction and snark pool reducers, only broadcast locally injected transactions and producer snarks. Libp2p layer takes care of diffs received from gossip already.
- **P2P**: Don't forget the initial peers list.
- WebRTC connection leaks.
- zkApp transaction proofs are now verified on a separate thread.
- Sometimes produced blocks were broadcasted too early.
- `OneOrTwo::zip` never panics now, on failure it returns an error.

### Changed

- On native, use jemalloc as the default allocator.
- Allocations reduced considerably by using stack-allocated bignums in the VRF evaluator.
- The same thread is now reused to verify all block proofs.
- `RUST_BACKTRACE` is always set to `full` now.

## [0.12.0] - 2024-12-04

### Fixed

- Properly handle time in cases in which the system goes to sleep.
- Various corner cases in block proof production.
- Improved ledgers sync during bootstrap (be smarter about which peers to query).
- **P2P**: More efficient memory usage when managing the p2p state.
- **P2P**: Lower outgoing traffic by being more conservative about what is broadcasted to each peer.
- **P2P**: Better handling of disconnections.
- **VRF**: Correctly handle cases in which the delegator table is empty.
- **Webnode**: Peer connection handling improvements.

### Changed

- **Webnode**: Reduced startup time by loading prover indexes in parallel to the bootstrap process.
- **Webnode**: Faster field operations (faster hashing and proving).
- Improved hashing performance by caching and reusing common prefixes.
- Added more pre-validation checks for blocks received from the network.

### Added

- **Webnode**: Transaction propagation
- **P2P**: Support for specifying the external ip to be advertised.

## [0.11.0] - 2024-10-31

### Added

- **Webnode**: Peer discovery and p2p based signaling (so that webnodes can find each other).
- **Webnode**: Connection authentication.
- Support for specifying external IPs.

### Fixed

- **Ledger**: Fixed a regression introduced in v0.10.0 that caused the ZKApp precondition checks ordering to not match the OCaml implementation's exactly, which resulted in block application failures.
- **Ledger**: Corrected handling of custom tokens in the block application logic.
- **P2P**: Reduced excessive outgoing traffic.
- **P2P**: Yamux fixes and improvements (backpressure).
- **Webnode**: Staging ledger sync timeout.

## [0.10.3] - 2024-10-16

### Added

- Support for name resolution in peer addresses.
- Block producer docker compose setup.
- WASM file containing the webnode is now included in the frontend image.
- Logging output to the filesystem in addition to stdout.
- Webnode: peer discovery and p2p based signaling.

## [0.10.0] - 2024-10-10

### Added

- GraphQL endpoints for o1js.
- Support for batched verification of proofs.
- Enabled full proof verification of completed works included in blocks.

### Changed

- Deduplication of zkApp logic.
- Various internal improvements.

### Fixed

- Regression in the verification zkApp transactions with feature flags (introduced when upgrading proof-systems).
- Potential overflow in yamux.
- Added a missing check for next epoch seed finality

## [0.9.0] - 2024-10-02

### Fixes

- Many bugfixes, performance, security and stability improvements.

## [0.8.14] - 2024-09-18

### Fixed

- Correctly show transaction fee values in the frontend.
- Make sure that incorrectly encoded finite field values are handled properly.

## [0.8.13] - 2024-09-18

### Fixed

- Many stability and security improvements.
- Make the JSON encodings of many kinds of values closer to what the Mina node produces to increase compatibility.

### Changed

- Combined all frontend docker images into a single one configurable at runtime.
- Many internal state machine refactorings.

## [0.8.3] - 2024-09-09

### Fixed

- Handling verification key updates with proof authorization kind.
- Block producer incorrectly discarding blocks if staking ledger between best tip and won slot were different.

## [0.8.2] - 2024-09-06

### Fixed

- Include circuit blobs in docker images, required for block production.
- Add missing bounds to ZkAppUri and TokenSymbol fields.
- Various stability improvements to make sure the node will not crash in certain circumstances.

### Changed

- Root snarked ledger re-syncs now reuse the previously in-progress root snarked ledger instead of starting again from the next-epoch ledger.
- Added `--libp2p-keypair=<path to json>` flag to specify encrypted secret key (with passphrase from `MINA_LIBP2P_PASS` environment variable).

## [0.8.1] - 2024-09-02

### Fixed

- Mempool: handling of missing verification key in the transaction pool.

## [0.8.0] - 2024-08-30

### Added

- Webnode: Streaming ledger sync RPC.
- P2P: Meshsub for gossip.
- P2P: Additional tests.

### Fixed

- Mempool: Various transaction pool issues.
- Poseidon hashing and witness generation when absorbing empty slices.

### Changed

- **Rust Toolchain**: Updated the minimum required Rust toolchain to version 1.80.

## [0.7.0] - 2024-08-02

### Added

- Transaction pool (alpha).
- Support for sending transactions, inspecting the transaction pool and the scan state to the block producer demo.

### Fixed

- P2P layer fixes and improvements.
- Various internal fixes and improvements.

### Changed

- **Rust Toolchain**: Updated the minimum required Rust toolchain to version 1.79.

## [0.6.0] - 2024-07-01

### Added

- Devnet support.
- Callbacks support in the state machine, support in reducer functions for queueing new actions.
- Cache for the genesis and initial epoch ledgers data for faster loading.

### Removed

- Berkeleynet support.

### Changed

- State machine records are now encoded in postcard format.

### Fixes

- General improvements in performance and stability.
- Various P2P layer issues.
- Correct handling of heartbeats in long-running P2P RPCs.
- Genesis snarked mask being overwritten, which sometimes resulted in some ledgers not being found when applying a block.
- State machine record-and-replay functionality issues.
- WASM target restored for the webnode.
- WebRTC P2P protocol restored for the webnode.

## [0.5.1] - 2024-06-01

### Added

- ARM docker builds.

## [0.5.0] - 2024-05-31

### Fixed

- When applying blocks, use the `supercharge_coinbase` value from the block which was being ignored before.
- Incorrect stream being used for RPC responses.
- Allow multiple nodes running on the same host to connect to each other.
- Invalid `delta_block_chain_proof` in block producer.
- Various p2p layer fixes.

### Added

- Support for PubSub in the p2p layer.
- Block producer dashboard, and simulator-based demo.
- Support for parsing `daemon.json` files with custom genesis ledgers.
- Chain ID computation (was hardcoded before).
- Multiple RPC and p2p tests.
- More limits to p2p messages, connections, and parsing.

### Removed

- Support for v1 messages in p2p layer.

## [0.4.0] - 2024-04-30

### Fixed

- Interactions with the ledger service are now async. This fixes situations in which expensive ledger operations could starve the state machine loop.
- Ledger synchronization issue that happens when synchronizing the node during the second epoch.
- Correctly handle the situation in which the best tip changes during staged ledger reconstruction, causing the reconstruct to produce a stale result.
- Various edge cases in p2p layer (error propagation, disconnection, self-connection).

### Added

- Support for `identify` protocol.
- P2P layer testing framework.
- Frontend: Block production page.

### Removed

- Removed rust-libp2p based code, in favor of our own libp2p implementation.

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

[Unreleased]: https://github.com/openmina/openmina/compare/v0.16.0...develop
[0.16.0]: https://github.com/openmina/openmina/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/openmina/openmina/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/openmina/openmina/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/openmina/openmina/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/openmina/openmina/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/openmina/openmina/compare/v0.10.3...v0.11.0
[0.10.3]: https://github.com/openmina/openmina/compare/v0.10.0...v0.10.3
[0.10.0]: https://github.com/openmina/openmina/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/openmina/openmina/compare/v0.8.14...v0.9.0
[0.8.14]: https://github.com/openmina/openmina/compare/v0.8.13...v0.8.14
[0.8.13]: https://github.com/openmina/openmina/compare/v0.8.3...v0.8.13
[0.8.3]: https://github.com/openmina/openmina/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/openmina/openmina/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/openmina/openmina/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/openmina/openmina/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/openmina/openmina/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/openmina/openmina/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/openmina/openmina/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/openmina/openmina/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/openmina/openmina/compare/v0.3.0...v0.4.0
[0.3.1]: https://github.com/openmina/openmina/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/openmina/openmina/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/openmina/openmina/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/openmina/openmina/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/openmina/openmina/releases/tag/v0.0.1

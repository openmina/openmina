# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- **Documentation**: add interactive OpenAPI documentation (Swagger UI, Stoplight
  Elements, Scalar) to the website, with `cargo xtask` for static spec generation
  ([#2119](https://github.com/o1-labs/mina-rust/pull/2119))
- **Node**: add OpenAPI documentation endpoints (`/api-docs/*`)
  ([#2119](https://github.com/o1-labs/mina-rust/pull/2119))

### Changed

- **Frontend**: remove Sentry error tracking dependencies and services
  ([#2130](https://github.com/o1-labs/mina-rust/pull/2130))
- **CI**: Add `merge_group` trigger to workflows for merge queue support
  ([#2127](https://github.com/o1-labs/mina-rust/pull/2127))
- **CI**: Add path filtering to skip Rust/backend workflows when only frontend
  files change ([#2115](https://github.com/o1-labs/mina-rust/pull/2115))
- Updated `proof-systems` dependencies (`kimchi`, `mina-curves`, `mina-hasher`, `mina-poseidon`, `mina-signer`, `o1-utils`, `poly-commitment`) from `0.2.0` to `0.3.0`.
- Adjusted ledger proofs implementation to align with `proof-systems` 0.3.0 API changes.
- Added `.gemini/` to `.gitignore`.
- Fix linter for latest beta Rust
  ([#2097](https://github.com/o1-labs/mina-rust/pull/2097))
- Bumped up `rsa` from `0.9.7` `0.9.10` and `numb-bigint-dig` from `0.8.4` to
  `0.8.6` ([#2096](https://github.com/o1-labs/mina-rust/pull/2096/))
- **Frontend**: remove old code adding so-called "deployment" information,
  leaking engineer information
  ([#2092](https://github.com/o1-labs/mina-rust/pull/2092))
- **Node**: migrate HTTP server from warp to axum
  ([#2119](https://github.com/o1-labs/mina-rust/pull/2119))

### Dependencies

- Bump react and react-dom from 19.2.3 to 19.2.4 in /website
  ([#2123](https://github.com/o1-labs/mina-rust/pull/2123),
  [#2124](https://github.com/o1-labs/mina-rust/pull/2124))
- Bump aws-config from 1.5.16 to 1.5.18
  ([#2125](https://github.com/o1-labs/mina-rust/pull/2125))
- Bump serde from 1.0.219 to 1.0.228, update private API usage for compatibility
  ([#2089](https://github.com/o1-labs/mina-rust/pull/2089))

## [0.19.0] - 2026-01-08

### Added

- **Documentation**: add Schnorr signatures specification from MinaProtocol/mina
  repository to Researchers section
  ([#1918](https://github.com/o1-labs/mina-rust/pull/1918))
- **Documentation**: add RPC API reference documentation with CI validation
  against o1Labs node, reorganize API docs into dedicated section
  ([#1745](https://github.com/o1-labs/mina-rust/pull/1745))
- **Node**: add top-level documentation for the crate `node`
  ([#1736](https://github.com/o1-labs/mina-rust/pull/1736))
- **CI**: automatically push Docker images for `vX.Y.Z` and `vX.Y` when tag `vX.Y.Z` is created
  ([#1838](https://github.com/o1-labs/mina-rust/pull/1838))
- **Web Node**: fix webnode build, add quality-of-life improvements when running
  webnode in Docker. ([#1778](https://github.com/o1-labs/mina-rust/pull/1778))
- **CI**: add beta channel lint workflow to catch build issues early
  ([#1875](https://github.com/o1-labs/mina-rust/pull/1875))
- **Feature**: Add logging if initial peers are invalid
  ([#1703](https://github.com/o1-labs/mina-rust/pull/1703))

### Fixed

- **Frontend**: Fix security vulnerabilities by upgrading Angular to 19.2.18
  to address XSS vulnerability in SVG script attributes and fix qs DoS
  vulnerability ([#2076](https://github.com/o1-labs/mina-rust/pull/2076))
- **Frontend**: Fixed a bug where user could reach a non-existent page
  with a non-existent block height on the scan state page
  ([#1966](https://github.com/o1-labs/mina-rust/pull/1966))
- **Frontend**: Fixed a bug where text was overlapping in the view.
  ([#1957](https://github.com/o1-labs/mina-rust/pull/1957))
- **CI**: fix version regex in build verification workflows to accept
  variable-length commit hashes (7+ characters) instead of exactly 7, adapting
  to Git's dynamic abbreviation based on repository size, fix
  [#1911](https://github.com/o1-labs/mina-rust/issues/1911)
  ([#1912](https://github.com/o1-labs/mina-rust/pull/1912))
- **Docker compose files**: replace old environment by local in
  docker-compose.block-producer.yml
  ([#1916](https://github.com/o1-labs/mina-rust/pull/1916)
- **CI**: Remove clippy exceptions for large enum/err variants added
  as part of the Rust 1.92 upgrade
  [#1968](https://github.com/o1-labs/mina-rust/pull/1968)

### Changes

- **Frontend**: Upgrade Angular from 19.x to 21.x, migrate templates from
  `*ngIf`/`*ngFor` to `@if`/`@for` control flow syntax, update NgRx to 19.x,
  and remove unused directive imports
  ([#2091](https://github.com/o1-labs/mina-rust/pull/2091))
- Bump webpack-bundle-analyzer from 4.9.0 to 5.1.1 in /frontend
  ([#1958](https://github.com/o1-labs/mina-rust/pull/1958))
- Bump webpack from 5.88.2 to 5.104.1 in /frontend
  ([#1960](https://github.com/o1-labs/mina-rust/pull/1960))
- Bump @sentry/angular from 8.35.0 to 10.32.1
  ([#1965](https://github.com/o1-labs/mina-rust/pull/1965))
- Bump typescript from 5.6.2 to 5.9.3 in /website
  ([#1888](https://github.com/o1-labs/mina-rust/pull/1888))
- **Build System**: rename crate packages to use `mina-` prefix consistently
  (`node` → `mina-node`, `p2p` → `mina-p2p`, `snark` → `mina-snark`,
  `vrf` → `mina-vrf`, `cli` → `mina-cli`) for clearer project identity, fix
  [#1902](https://github.com/o1-labs/mina-rust/issues/1902)
  ([#1925](https://github.com/o1-labs/mina-rust/pull/1925))
- **Frontend**: Renamed `@openmina/shared` to `@mina-rust/shared` and all its references
  ([#1953](https://github.com/o1-labs/mina-rust/pull/1953)
- **Frontend**: Renamed `@openmina/shared` to `@mina-rust/shared` and all its references
  ([#1953](https://github.com/o1-labs/mina-rust/pull/1953)
- **Repository Structure**: reorganize workspace into `crates/`, `libs/`,
  `vendor/`, and `tools/` directories for clearer separation of concerns
  ([#1910](https://github.com/o1-labs/mina-rust/pull/1910))
- **Dependencies**: vendor alloc-test into `vendor/alloc-test` to centralize all
  code in the monorepo ([#1815](https://github.com/o1-labs/mina-rust/pull/1815))
- **Dependencies**: vendor redux-rs into `vendor/redux` to centralize all code
  in the monorepo ([#1814](https://github.com/o1-labs/mina-rust/pull/1814))
- **Build System**: enforce GNU sed on macOS, require `brew install gnu-sed`
  for `make fix-trailing-whitespace`, add CI test for cross-platform support,
  fix [#1864](https://github.com/o1-labs/mina-rust/issues/1864)
  ([#1898](https://github.com/o1-labs/mina-rust/pull/1898),
  [#1872](https://github.com/o1-labs/mina-rust/pull/1872)).
  ([#1872](https://github.com/o1-labs/mina-rust/pull/1872))
- **Frontend**: Move `@openmina/shared` and `openmina-styles` from npm packages
  to vendor directory as local file dependencies, simplifying frontend
  dependency management and consolidating all frontend code in the monorepo
  ([#1799](https://github.com/o1-labs/openmina/pull/1799))
- **Dependency**: use tag instead of references of o1-labs/proof-systems, fix
  [[#1674](https://github.com/o1-labs/mina-rust/issues/1674)]
  ([#1673](https://github.com/o1-labs/mina-rust/pull/1673))
- Remove ocaml-interop dependency, fix
  [#1235](https://github.com/o1-labs/mina-rust/issues/1235)
  ([#1646](https://github.com/o1-labs/mina-rust/pull/1646))
- *Build** Update Rust to 1.92 [#1894](https://github.com/o1-labs/mina-rust/issues/1894)
- *CI*: run builds on macos-latest for each patch submission, fixing
  [#1899](https://github.com/o1-labs/mina-rust/issues/1899)
  ([#1900](https://github.com/o1-labs/mina-rust/pull/1900))
- **Docs**: add notes in `.cargo/config.toml` about the magic RUSTFLAGS for WASM
  [#1863](https://github.com/o1-labs/mina-rust/issues/1863)
- **Docs**: Consistent naming of the "web node" across docs
  [#1858](https://github.com/o1-labs/mina-rust/issues/1858)
- **Docs**: Reorganized web node docs with clear audience split
  [#2006](https://github.com/o1-labs/mina-rust/pull/2006)
- **Web Node**: Removed compatible browser gate. Web Node is experimental anyway
  [#2006](https://github.com/o1-labs/mina-rust/pull/2006)
- **Dependencies**: bump up tokio from 1.26.0 to 1.46.1
  ([#2053](https://github.com/o1-labs/mina-rust/pull/2053))
- **Web Node**: Standardized WebRTC peer address to Multiaddr and deprecated
  old "WebRTC-Multiaddrish" format [#2052](https://github.com/o1-labs/mina-rust/pull/2052)

### Removed

- **Ledger**: remove FFI codebase and `port_ocaml` module, fix
  [#1235](https://github.com/o1-labs/mina-rust/issues/1235)
- **Build**: remove unused Docker files from `tools/testing/docker` and
  corresponding Makefile targets
- **Frontend**: remove outdated `frontend/functions` directory containing unused
  Firebase Cloud Functions
- **CI**: remove network debugger from CI
  ([#1700](https://github.com/o1-labs/mina-rust/pull/1700))
- **CI**: remove setup-ocaml from CI
  ([#1879](https://github.com/o1-labs/mina-rust/pull/1879))
- **Web Node**: Removed web-node-secrets.json format and unified key handling
  across documentation to use standard encrypted key format
  ([#1917](https://github.com/o1-labs/mina-rust/issues/1971))
- **CLI** Removed `-web-node-secrets` flag from `mina misc mina-key-pair`
  ([#1917](https://github.com/o1-labs/mina-rust/issues/1971))


## [0.18.1] - 2025-11-20

### Added

- **Documentation**: Add comprehensive API endpoints reference for the Node
  Dashboard, documenting all endpoints and specific data fields used by the
  frontend ([#1566](https://github.com/o1-labs/mina-rust/issues/1566))

### Fixed

- **Docker Compose**: Fix frontend black screen issue by changing environment
  from `compose` to `local` and exposing port 3000 for rust node HTTP API (hotfix `v0.18.1`)
  from `compose` to `local` and exposing port 3000 for mina-node HTTP API
  ([#1649](https://github.com/o1-labs/mina-rust/pull/1649))

### Changed

- **Dependencies/proof-systems**: bump up proof-systems to 282faf5
  ([#1662](https://github.com/o1-labs/mina-rust/pull/1662))
- **Tests**: removed unused tests and fixed tests
  ([#1682](https://github.com/o1-labs/mina-rust/pull/1682))

## [0.18.0] - 2025-11-04

### OCaml node

- Update the CI and code to compare with the latest release 3.3.0-alpha1-6929a7e
  ([#1454](https://github.com/o1-labs/mina-rust/pull/1454))

### Added

- **Frontend**: add production seed nodes for the production environment
  ([#1837](https://github.com/o1-labs/mina-rust/issues/1837)
- **Website**: Update Docusaurus to version 3.9.2 from 3.9.1 for latest
  features and bug fixes
  ([#1583](https://github.com/o1-labs/mina-rust/pull/1583))
- **Website**: Migrate deprecated `onBrokenMarkdownLinks` configuration option
  to the new `markdown.hooks.onBrokenMarkdownLinks` format for Docusaurus v4
  compatibility ([#1583](https://github.com/o1-labs/mina-rust/pull/1583))
- **Development Tools**: Add `setup-taplo` and `setup` Makefile targets to
  simplify development environment setup. Update `release-validate` script to
  test only packages that are tested in CI, avoiding untested packages with
  failing tests
  ([#1573](https://github.com/o1-labs/mina-rust/pull/1573))
- **CI**: Add validation workflows for block producer nodes infrastructure,
  including connectivity and API capability testing similar to plain nodes
  ([#1571](https://github.com/o1-labs/mina-rust/pull/1571))
- **Website**: Add block producer nodes documentation page with GraphQL query
  examples for latest canonical block and transaction information. Include note
  that block production functionality is under development
  ([#1571](https://github.com/o1-labs/mina-rust/pull/1571))
- **CLI**: add GraphQL introspection and execution commands under `mina
  internal graphql`. Three new commands enable dynamic endpoint discovery
  (`list`), detailed schema inspection (`inspect <endpoint>`), and arbitrary
  query execution (`run [query]`) with support for multiple input methods
  (command line, stdin, file) and variables. All commands use GraphQL
  introspection API for dynamic discovery without hardcoded endpoints. CI tests
  validate commands against o1Labs infrastructure
  ([#1565](https://github.com/o1-labs/mina-rust/pull/1565))
- **CI**: add devnet and mainnet Caml nodes to remote GraphQL test suite to
  ensure compatibility between Rust and OCaml implementations
  ([#1542](https://github.com/o1-labs/mina-rust/pull/1542))
- **CI**: add workflow to test GraphQL queries with the OCaml node
  ([#1465](https://github.com/o1-labs/mina-rust/pull/1465))
- **Website**: add o1Labs infrastructure entry, describing the nodes managed by
  o1Labs, including seed and plain nodes
  ([#1430](https://github.com/o1-labs/mina-rust/pull/1430)). The infrastructure
  is tested accordingly in the CI at every push.
- **Website**: add comparison list of implemented and missing GraphQL endpoints
  ([#1486](https://github.com/o1-labs/mina-rust/pull/1466))
- **Frontend**: use a Makefile for scripts in package.json
  ([#1468](https://github.com/o1-labs/mina-rust/pull/1468))
- **Frontend**: define one Makefile target per build configuration
  ([#1469](https://github.com/o1-labs/mina-rust/pull/1469))
- **Frontend**: revamping the Docker image, and provide a better support and
  documentation for the different configuration.
  ([#1473](https://github.com/o1-labs/mina-rust/pull/1473))
- **Website**: add "Join devnet" page with instructions for community members to
  join the devnet program and test the Rust node implementation
  ([#1425](https://github.com/o1-labs/mina-rust/issues/1425))
- **Website**: add documentation guidelines
  ([#1493](https://github.com/o1-labs/mina-rust/pull/1493))
- **Website**: add search bar with Algolia
  ([#1493](https://github.com/o1-labs/mina-rust/pull/1509))
- **Ledger/ZkAppCommand**: show how to build a ZkAppCommand from scratch, with
  dummy values ([#1514](https://github.com/o1-labs/mina-rust/pull/1514))
- **CI/Documentation**: add a script to check the references to the OCaml code
  ([#1525](https://github.com/o1-labs/mina-rust/pull/1525)).
- **mina-node-account**: move tests into `node/account/tests`, document the
  library and run the tests in CI
  ([#1540](https://github.com/o1-labs/mina-rust/pull/1540)).
- **Ledger**: add tests to verify some properties transaction application
  should have on the ledger. Also, document the different types of transactions
  that can be used to modify the ledger
([#1541](https://github.com/o1-labs/mina-rust/pull/1541))
- **CLI**: introduce a subcommand `wallet` to be able to send transactions, get
  the public key and address from the secret key, get balance and generate
  wallets from the CLI. End-to-end tests are added in the CI and a new target
  `make test-wallet` has been added. This focuses on basic payments
  ([#1543](https://github.com/o1-labs/mina-rust/pull/1543))
- **Makefile**: add targets to only build and run tests of the package
  `mina-node-native` ([#1549](https://github.com/o1-labs/mina-rust/pull/1549))
- **CI**: add a step in tests to run the unit/integration tests of the package
  `mina-node-native` ([#1549](https://github.com/o1-labs/mina-rust/pull/1549))
- **Tests**: add account creation test cases for payment and coinbase
  transactions, verifying correct handling of account creation fees during
  the first pass of transaction application
  ([#1581](https://github.com/o1-labs/mina-rust/pull/1581))
- **tools**: remove stack allocation from tools
  ([#1576](https://github.com/o1-labs/mina-rust/pull/1576))

### Changed

- **Development Tools**: Update taplo configuration to exclude `node_modules` and
  `target` directories from TOML formatting checks
  ([#1573](https://github.com/o1-labs/mina-rust/pull/1573))
- **Build System**: Move salsa-simple from tools/ to vendor/ and alphabetize
  workspace members. salsa-simple is a vendored XSalsa20 implementation with
  serde support, not a command-line tool
  ([#1580](https://github.com/o1-labs/mina-rust/pull/1580))
- **CI**: Speed up CI by decoupling test runs from full build completion.
  Created dedicated single-platform build jobs that run only on ubuntu-22.04
  to produce artifacts needed for testing, allowing tests to start as soon as
  those builds complete instead of waiting for all cross-platform matrix builds
  ([#1427](https://github.com/o1-labs/mina-rust/issues/1427))
- **CI**: Update CI to test on specific macOS versions (13, 14, 15) instead of
  only macos-latest, providing better coverage across macOS versions that
  developers are using ([#1421](https://github.com/o1-labs/mina-rust/pull/1421))
- **Website**: update GraphQL API page by extracting all endpoints and run it in
  CI, at every push, with the o1Labs managed plain nodes
  ([#1421](https://github.com/o1-labs/mina-rust/pull/1421)).
- **Frontend**: rename all occurences of openmina.scss to mina-rust.scss
  ([#1467](https://github.com/o1-labs/mina-rust/pull/1467))
- **Website**: bump up to docusaurus 3.9.1
  ([#1500](https://github.com/o1-labs/mina-rust/pull/1500))
- **Dependencies**: move all dependencies to the workspace Cargo.toml
  ([#1513](https://github.com/o1-labs/mina-rust/pull/1513))
- **scan_state**: refactorize `transaction_logic.rs` in smaller modules
  ([#1515](https://github.com/o1-labs/mina-rust/pull/1515))
- **CI/tests**: run the step `build-wasm` for each push and PR to `develop`
  ([#1497](https://github.com/o1-labs/mina-rust/pull/1497))
- **ledger/scan_state/transaction_logic**: update OCaml references in `mod.rs`
  ([#1525](https://github.com/o1-labs/mina-rust/pull/1525))
- **ledger/scan_state/transaction_logic**: move submodule `for_tests` into a
  new file `zkapp_command/for_tests.rs`
  ([#1527](https://github.com/o1-labs/mina-rust/pull/1527)).
- **Ledger/scan-state/transaction-logic**: split
  `ledger::scan_state::transaction_logic::zkapp_command` into submodules in a
  new directory `zkapp_command`
  ([#1528](https://github.com/o1-labs/mina-rust/pull/1528/))
- **Codebase**: remove unused `ledger/src/poseidon/fp.rs` file
  ([#1538](https://github.com/o1-labs/mina-rust/pull/1538))
- **Ledger**: convert unused binary to proper criterion benchmarks. Benchmarks
  are now built as part of the existing build workflows across all platforms,
  reusing build caches for efficiency
  ([#1539](https://github.com/o1-labs/mina-rust/pull/1539))
- **Ledger**: document, clean and add tests for the crate `mina-tree`
  ([#1531](https://github.com/o1-labs/mina-rust/pull/1531).
- **GraphQL**: fixed parsing when neither signature nor proof is given.
  See issue [#1464](https://github.com/o1-labs/mina-rust/issues/1464).
  Fixed in [#1546](https://github.com/o1-labs/mina-rust/pull/1546/)
  ([#1546](https://github.com/o1-labs/mina-rust/pull/1546))
- **transactions**: fixed a typo in `p2p_request_transactions_if_needed` where snark count was used instead of transaction count
  ([#1563](https://github.com/o1-labs/mina-rust/pull/1563)).
- **tools**: remove heartbeat-processors
  ([#1577](https://github.com/o1-labs/mina-rust/pull/1577))
- **tools**: remove producer-dashboard
  ([#1578](https://github.com/o1-labs/mina-rust/pull/1578))
- **CI**: build benches for each PR with the workflow `tests`, and fix the step
  in the workflow `build` by adding the missing SQLx/SQLite setup
  ([#1548](https://github.com/o1-labs/mina-rust/pull/1548))
- **frontend**: remove leaderboard from the frontend
  ([#1579](https://github.com/o1-labs/mina-rust/pull/1579)

### Fixed

- **Development Tools**: Fix `fix-trailing-whitespace` Makefile target to work
  correctly on macOS by removing conflicting `-e` flag in sed command
  ([#1573](https://github.com/o1-labs/mina-rust/pull/1573))

## [0.17.0] - 2025-04-08

### OCaml node

- Update the CI and code to compare with the latest release 3.2.0-alpha1-7f94ae0
  ([#1236](https://github.com/o1-labs/openmina/pull/1236), fix
  [#1158](https://github.com/o1-labs/openmina/issues/1158))
- Update the CI and code to compare with the latest release 3.2.0-beta1-978866c
  ([#1250](https://github.com/o1-labs/openmina/pull/1250))
- Update the CI and code to compare with the latest release 3.2.0-beta2-939b08d
  ([#1285](https://github.com/o1-labs/openmina/pull/1285))


### Added

- **Documentation**: Add instructions to update the OCaml node dependencies.
  ([#1237](https://github.com/o1-labs/openmina/pull/1237))
- **Development Tools**: Automation script for updating OCaml node references
  with comprehensive documentation and CI testing to ensure reproducibility
  ([#1252](https://github.com/o1-labs/openmina/pull/1252))
- **Documentation**: Complete Docusaurus-based documentation website with
  comprehensive guides for node runners, developers, and researchers
  ([#1234](https://github.com/o1-labs/openmina/pull/1234)). It migrates all the
  documentation from `docs` to `website/docs`. It includes SEO metadata and
  social media integration.
- **Documentation**: Developer getting-started guide with automated setup
  scripts for system dependencies, Rust toolchain, Node.js, Docker, and WASM
  tools. This is run nightly and on-demand via PR labels to avoid blocking
  development workflow. ([#1234](https://github.com/o1-labs/openmina/pull/1234)).
- **Development Tools**: Enhanced formatting infrastructure with MDX file
  support and trailing whitespace management
  ([#1234](https://github.com/o1-labs/openmina/pull/1234))
- **Documentation**: add instructions for developers using MacOS based systems,
  and test the installation commands in CI
  ([#1247](https://github.com/o1-labs/openmina/pull/1234)).
- **Development tools**: Add shellcheck to the CI
  ([#1255](https://github.com/o1-labs/openmina/pull/1255))
- **Makefile**: Add `run-block-producer` target with devnet/mainnet support and
  `generate-block-producer-key` target for key generation
  ([#1221](https://github.com/o1-labs/openmina/pull/1221)).
- **Documentation**: add section regarding peers setup and seeds
  ([#1295](https://github.com/o1-labs/openmina/pull/1295))
- **Node**: add `openmina misc mina-encrypted-key` to generate a new encrypted
  key with password, as the OCaml node provides
  ([#1284](https://github.com/o1-labs/openmina/pull/1284/)).
- **CI**: push images on tags and develop to
  [o1labs/mina-rust-frontend](https://hub.docker.com/r/o1labs/mina-rust-frontend)
  and [o1labs/mina-rust](https://hub.docker.com/r/o1labs/mina-rust), and support
  ARM and AMD64, fixing
  [!1336](https://github.com/o1-labs/mina-rust/issues/1336),
  [!1192](https://github.com/o1-labs/mina-rust/issues/1192)
  ([#1337](https://github.com/o1-labs/mina-rust/pull/1337),
  [#1335](https://github.com/o1-labs/mina-rust/pull/1335),
  [#1344](https://github.com/o1-labs/mina-rust/pull/1344),
  [#1340](https://github.com/o1-labs/mina-rust/pull/1340),
  [#2981ecf](https://github.com/o1-labs/mina-rust/commit/2981ecf9519517f69947838407a09ea96f7ff293),
  [#1e9f33f](https://github.com/o1-labs/mina-rust/commit/1e9f33f125dcaab36d9dd2dc17759e3476bc36e8))
- **Documentation**: add release communication checklist
  ([#1390](https://github.com/o1-labs/mina-rust/pull/1390))

### Changed

- **CI**: Generalized build jobs to support multiple platforms (Ubuntu 22.04,
  Ubuntu 24.04, Ubuntu 24.04 ARM, macOS latest), updated test execution to
  ubuntu-latest, and fixed GLIBC compatibility for scenario tests by using
  Ubuntu 22.04 for test binary artifacts to ensure compatibility with Debian
  Bullseye container environment ([#1249](https://github.com/o1-labs/openmina/pull/1249))
- **Build System**: Enhanced Makefile with documentation-related targets and
  comprehensive formatting commands
  ([#1234](https://github.com/o1-labs/openmina/pull/1234))
- Use consistent `use` statements for fields. Replace `mina_hasher::Fp` with
  `mina_curves::pasta::Fp`.
  ([#1269](https://github.com/o1-labs/openmina/pull/1269/)).
- **Proof systems**: Updated proof systems to use same version as OCaml node
- **CI**: set fail-fast to false to prevent cancellation of other jobs
  ([#1305](https://github.com/o1-labs/openmina/pull/1305))
- **Website**: (temporary) new design, for a first release and rename OpenMina
  to "the Mina Russt node"
  ([#1312](https://github.com/o1-labs/openmina/pull/1312)).
- **Codebase**: rename `openmina` to `mina`, the project being officially
  called for now "the Mina Rust node"
  ([#1314](https://github.com/o1-labs/openmina/pull/1312)).
- **Dependencies**: update redux-rs to o1-labs fork, after we moved from
  openmina to o1-labs organization
  ([#1356](https://github.com/o1-labs/mina-rust/pull/1356)).

### Fixed

- **Documentation**: Resolved broken links and bare URL warnings in Rust
  documentation ([#1234](https://github.com/o1-labs/openmina/pull/1234))
- **Build System**: Fixed SQLx compilation errors in lint workflow
  ([#1234](https://github.com/o1-labs/openmina/pull/1234))
- **Formatting**: Standardized markdown and MDX files to 80-character line
  wrapping ([#1234](https://github.com/o1-labs/openmina/pull/1234))
- **Build warnings**: Fix the warnings generated by the missing lifetime
  ([#1244](https://github.com/o1-labs/openmina/pull/1244))

### Dependencies

- bump proc-macro2 from 1.0.93 to 1.0.95
  ([#1230](https://github.com/o1-labs/openmina/pull/1230))
- bump itertools from 0.10.5 to 0.12.0 #1228
  ([#1228](https://github.com/o1-labs/openmina/pull/1228))
- bump aes from 0.8.3 to 0.8.4
  ([#1272](https://github.com/o1-labs/openmina/pull/1272))
- Remove OpenMina forks of arkworks and proof-systems
  ([#1383](https://github.com/o1-labs/mina-rust/pull/1383))
- Bump mio from 1.0.3 to 1.0.4
  ([#1384](https://github.com/o1-labs/mina-rust/pull/1384))

### Other

- Added a `flake.nix` for building the project on nixos/using nix ([#1241](https://github.com/o1-labs/openmina/pull/1241)),
  together with **documentation**: instructions for developers,
  and test the installation commands in CI.


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

[Unreleased]: https://github.com/openmina/openmina/compare/v0.18.1...develop
[0.18.1]: https://github.com/openmina/openmina/compare/v0.18.0...v0.18.1
[0.18.0]: https://github.com/openmina/openmina/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/openmina/openmina/compare/v0.16.0...v0.17.0
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

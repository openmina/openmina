# Shadow Testing Epic - Sub-Issues Breakdown

This document outlines the sub-issues for implementing comprehensive shadow
testing infrastructure to validate the Rust node implementation against the
OCaml reference implementation.

**Epic:**
[#1779 - Shadow Testing/Shadowing Strategy](https://github.com/o1-labs/mina-rust/issues/1779)

## Overview

Shadow testing enables running the Rust node alongside the OCaml implementation,
processing identical network data and comparing outputs to validate behavioral
equivalence without affecting production systems.

The work is organized into 6 phases:

1. **Foundation** - Core infrastructure and unified interfaces
2. **Ledger validation** - Deep ledger state comparison
3. **Continuous monitoring** - Real-time divergence detection
4. **Advanced analysis** - Action-level tracking and diagnostics
5. **Comprehensive scenarios** - Test scenario library
6. **CI integration** - Automated testing in development workflow

---

## Phase 1: Foundation

### Issue 1.1: Dual-Node Test Harness

**Title:** Shadow testing: Create dual-node test harness for coordinated
Rust/OCaml testing

**Description:**

Extend the existing `Cluster` infrastructure in `tools/testing/` to support
coordinated testing of both Rust and OCaml nodes with identical configurations.

**Acceptance criteria:**

- [ ] Create `DualNodeHarness` struct that manages both node types
- [ ] Implement configuration translation (Rust config <-> OCaml config)
- [ ] Support spawning both nodes with identical genesis and network settings
- [ ] Handle lifecycle management (start, stop, restart, cleanup)
- [ ] Provide unified log collection from both nodes
- [ ] Support both local binary and Docker-based OCaml nodes

**Technical notes:**

- Build on existing `OcamlNode` wrapper in `tools/testing/src/node/ocaml/`
- Leverage `Cluster` patterns from `tools/testing/src/cluster/`
- Ensure port allocation doesn't conflict between implementations

**Labels:** `shadow-testing`, `infrastructure`, `phase-1`

---

### Issue 1.2: Unified Query Interface

**Title:** Shadow testing: Implement unified query interface for Rust and OCaml
nodes

**Description:**

Create an abstraction layer that provides consistent query capabilities across
both implementations, abstracting over GraphQL (OCaml) and internal RPC/state
access (Rust).

**Acceptance criteria:**

- [ ] Define `NodeQueryInterface` trait with common query methods
- [ ] Implement for Rust nodes using internal state access
- [ ] Implement for OCaml nodes using GraphQL
- [ ] Support querying:
  - Sync status
  - Best chain (block hashes)
  - Genesis constants
  - Daemon status
  - Network peers
- [ ] Handle timeout and retry logic consistently
- [ ] Provide async interface compatible with tokio

**Technical notes:**

- Reference existing GraphQL queries in `tools/archive-breadcrumb-compare/`
- Consider caching for frequently-queried values
- Handle version differences in GraphQL schemas gracefully

**Labels:** `shadow-testing`, `infrastructure`, `phase-1`

---

### Issue 1.3: State Serialization for Comparison

**Title:** Shadow testing: Implement state extraction and serialization
utilities

**Description:**

Create utilities to extract and serialize node state from both implementations
into a canonical format suitable for comparison.

**Acceptance criteria:**

- [ ] Define canonical comparison schema for key state types
- [ ] Implement state extraction for Rust nodes (account state, block info)
- [ ] Implement state extraction for OCaml nodes via GraphQL
- [ ] Create serialization format that normalizes ordering differences
- [ ] Handle protocol state proofs specially (semantic vs binary equality)
- [ ] Support partial state extraction (specific accounts, blocks)

**Technical notes:**

- Use JSON as intermediate format for debugging/inspection
- Consider binary format for performance in automated tests
- Reference `mina-p2p-messages` types for common structures

**Labels:** `shadow-testing`, `infrastructure`, `phase-1`

---

### Issue 1.4: Basic Snapshot Comparison Tool

**Title:** Shadow testing: Create basic state snapshot comparison tool

**Description:**

Build a command-line tool that captures and compares state snapshots from both
implementations, reporting differences in a structured format.

**Acceptance criteria:**

- [ ] CLI tool that connects to both Rust and OCaml nodes
- [ ] Capture best chain and compare block hashes
- [ ] Capture sync status and compare
- [ ] Report mismatches with detailed diff output
- [ ] Support JSON and human-readable output formats
- [ ] Exit with appropriate status codes for CI use

**Technical notes:**

- Can extend or parallel `tools/archive-breadcrumb-compare/`
- Use unified query interface from Issue 1.2
- Focus on quick validation, not exhaustive comparison

**Labels:** `shadow-testing`, `tooling`, `phase-1`

---

## Phase 2: Ledger Validation

### Issue 2.1: Account-Level Comparison

**Title:** Shadow testing: Implement account-level ledger comparison

**Description:**

Create infrastructure to compare individual account states between Rust and
OCaml implementations, validating balance, nonce, and other account properties.

**Acceptance criteria:**

- [ ] Query account state from both implementations by public key
- [ ] Compare:
  - Balance
  - Nonce
  - Receipt chain hash
  - Delegate
  - Token holdings (for non-default tokens)
  - Timing/vesting information
  - zkApp state (if applicable)
- [ ] Handle accounts that exist in one implementation but not other
- [ ] Report detailed differences with account context

**Technical notes:**

- OCaml: Use `account` GraphQL query
- Rust: Direct ledger access through testing service
- Consider sampling strategy for large ledgers

**Labels:** `shadow-testing`, `ledger`, `phase-2`

---

### Issue 2.2: Ledger Hash Validation

**Title:** Shadow testing: Implement Merkle tree hash validation for ledgers

**Description:**

Validate that ledger Merkle roots and intermediate hashes match between
implementations, ensuring structural consistency.

**Acceptance criteria:**

- [ ] Compare snarked ledger hash from both implementations
- [ ] Compare staged ledger hash
- [ ] Validate at epoch boundaries (staking, next epoch ledgers)
- [ ] Report first divergence point in tree if hashes differ
- [ ] Handle pending transactions that may cause temporary divergence

**Technical notes:**

- Ledger hashes available via GraphQL `protocolState` queries
- Root hash comparison is fast; deeper tree comparison needs more work
- Consider caching known-good hashes

**Labels:** `shadow-testing`, `ledger`, `phase-2`

---

### Issue 2.3: Transaction Pool Comparison

**Title:** Shadow testing: Implement transaction pool state comparison

**Description:**

Compare pending transaction pools between implementations to validate that both
nodes accept and retain the same transactions.

**Acceptance criteria:**

- [ ] Extract pending transactions from both implementations
- [ ] Compare by transaction hash
- [ ] Identify transactions in one pool but not other
- [ ] Validate fee ordering is consistent
- [ ] Track transaction lifecycle (added, dropped, included)

**Technical notes:**

- OCaml: Use `pooledUserCommands` GraphQL query
- Rust: Access via testing service RPC
- Transaction ordering may differ while contents match

**Labels:** `shadow-testing`, `transaction-pool`, `phase-2`

---

### Issue 2.4: Scan State Validation

**Title:** Shadow testing: Implement scan state and staged ledger validation

**Description:**

Compare scan state trees and staged ledger state between implementations to
validate SNARK work tracking and pending transaction application.

**Acceptance criteria:**

- [ ] Compare scan state tree structure
- [ ] Validate pending coinbase stacks match
- [ ] Compare fee excess calculations
- [ ] Validate SNARK work availability tracking
- [ ] Report scan state divergences with position information

**Technical notes:**

- Scan state is complex; may need to compare at higher level first
- Consider comparing only at block boundaries initially
- Fee excess is critical for protocol correctness

**Labels:** `shadow-testing`, `staged-ledger`, `phase-2`

---

## Phase 3: Continuous Monitoring

### Issue 3.1: Periodic Snapshot Service

**Title:** Shadow testing: Implement periodic state snapshot service

**Description:**

Create a background service that periodically captures state snapshots from both
implementations during long-running tests, enabling post-hoc analysis.

**Acceptance criteria:**

- [ ] Configurable snapshot interval (by time or block count)
- [ ] Capture minimal state needed for divergence detection
- [ ] Store snapshots with timestamps and block heights
- [ ] Support configurable storage backend (filesystem, database)
- [ ] Minimize performance impact on nodes under test

**Technical notes:**

- Snapshots should be queryable by block height
- Consider compression for storage efficiency
- Include metadata (node versions, config hashes)

**Labels:** `shadow-testing`, `monitoring`, `phase-3`

---

### Issue 3.2: Divergence Detection Engine

**Title:** Shadow testing: Implement real-time divergence detection

**Description:**

Build an engine that continuously monitors both implementations and detects when
they diverge, providing early warning and context for analysis.

**Acceptance criteria:**

- [ ] Monitor best chain tip from both nodes
- [ ] Detect block hash mismatches immediately
- [ ] Track fork depth if implementations follow different chains
- [ ] Emit alerts/events on divergence detection
- [ ] Record state at divergence point for later analysis
- [ ] Support configurable detection sensitivity

**Technical notes:**

- Use lightweight polling for continuous monitoring
- Consider WebSocket subscriptions if available
- False positives possible during reorgs; add debouncing

**Labels:** `shadow-testing`, `monitoring`, `phase-3`

---

### Issue 3.3: Diff Storage and Analysis

**Title:** Shadow testing: Implement diff storage and historical analysis

**Description:**

Create infrastructure to store comparison diffs over time and support historical
analysis of divergences.

**Acceptance criteria:**

- [ ] Store diffs in structured format (JSON, SQLite, or similar)
- [ ] Index by block height, timestamp, and divergence type
- [ ] Support querying historical divergences
- [ ] Track divergence patterns over time
- [ ] Export for external analysis tools

**Technical notes:**

- Consider SQLite for single-node tests, PostgreSQL for distributed
- Include git commit hash in metadata for regression tracking
- Support diff aggregation and summarization

**Labels:** `shadow-testing`, `monitoring`, `phase-3`

---

### Issue 3.4: Regression Tracking System

**Title:** Shadow testing: Implement regression tracking and baseline comparison

**Description:**

Build a system to track known-good baselines and detect when new changes
introduce divergences from previously-passing states.

**Acceptance criteria:**

- [ ] Store baseline snapshots from successful test runs
- [ ] Compare new runs against established baselines
- [ ] Identify which commit introduced a regression
- [ ] Support baseline versioning and updates
- [ ] Generate regression reports for PRs

**Technical notes:**

- Integrate with git for commit tracking
- Store baselines in reproducible format
- Consider artifact storage for large baselines

**Labels:** `shadow-testing`, `monitoring`, `ci`, `phase-3`

---

## Phase 4: Advanced Analysis

### Issue 4.1: Action-Level Tracking

**Title:** Shadow testing: Implement action-level execution tracking

**Description:**

Create infrastructure to track and correlate actions executed by both
implementations, enabling fine-grained divergence analysis.

**Acceptance criteria:**

- [ ] Record actions from Rust node (using existing state machine)
- [ ] Infer actions from OCaml node state changes
- [ ] Correlate corresponding actions between implementations
- [ ] Identify first diverging action in sequence
- [ ] Support action filtering and aggregation

**Technical notes:**

- Rust actions are directly observable via state machine
- OCaml actions must be inferred from state delta
- Focus on high-level actions initially (block application, tx processing)

**Labels:** `shadow-testing`, `analysis`, `phase-4`

---

### Issue 4.2: State Transition Validation

**Title:** Shadow testing: Implement state transition validation framework

**Description:**

Validate that identical inputs produce identical state transitions in both
implementations, beyond just final state comparison.

**Acceptance criteria:**

- [ ] Define state transition equivalence criteria
- [ ] Compare state deltas from same input
- [ ] Track intermediate state during block application
- [ ] Validate effect ordering (within tolerance)
- [ ] Report transition-level divergences

**Technical notes:**

- State transitions may have valid ordering variations
- Focus on semantic equivalence, not implementation details
- Consider recording state before/after each major operation

**Labels:** `shadow-testing`, `analysis`, `phase-4`

---

### Issue 4.3: Failure Diagnostics Framework

**Title:** Shadow testing: Build failure diagnostics and root cause analysis
tools

**Description:**

Create tools to diagnose why implementations diverged and identify the root
cause of discrepancies.

**Acceptance criteria:**

- [ ] Generate comprehensive state dump at divergence point
- [ ] Provide diff visualization (text, JSON diff, or web UI)
- [ ] Trace transaction effects through both implementations
- [ ] Link divergences to specific code paths (when possible)
- [ ] Generate diagnostic reports for debugging

**Technical notes:**

- State dumps should be self-contained and reproducible
- Consider integration with existing debug tooling
- Include network message traces if available

**Labels:** `shadow-testing`, `analysis`, `debugging`, `phase-4`

---

### Issue 4.4: Comparative Replay System

**Title:** Shadow testing: Implement comparative replay with detailed tracing

**Description:**

Build capability to replay recorded sequences through both implementations with
detailed tracing to identify exact point and cause of divergence.

**Acceptance criteria:**

- [ ] Record input sequences (blocks, transactions, network messages)
- [ ] Replay through both implementations
- [ ] Capture detailed trace during replay
- [ ] Compare traces to identify first difference
- [ ] Support stepping through replay for debugging

**Technical notes:**

- Leverage existing scenario recording infrastructure
- Traces should be comparable despite implementation differences
- Consider deterministic replay mode for Rust node

**Labels:** `shadow-testing`, `analysis`, `phase-4`

---

## Phase 5: Comprehensive Scenarios

### Issue 5.1: Bootstrap and Sync Scenarios

**Title:** Shadow testing: Implement bootstrap and synchronization test
scenarios

**Description:**

Create test scenarios validating that both implementations bootstrap and
synchronize identically under various conditions.

**Acceptance criteria:**

- [ ] Both nodes starting from same genesis
- [ ] One node bootstrapping from the other
- [ ] Bootstrap from network with existing chain
- [ ] Sync after network partition
- [ ] Catchup scenarios with varying chain lengths

**Technical notes:**

- Use existing scenario framework in `tools/testing/`
- Consider parameterized scenarios for coverage
- Document expected behaviors and invariants

**Labels:** `shadow-testing`, `scenarios`, `phase-5`

---

### Issue 5.2: Block Production Scenarios

**Title:** Shadow testing: Implement block production and consensus test
scenarios

**Description:**

Create scenarios validating block production and consensus state evolution
across both implementations.

**Acceptance criteria:**

- [ ] Both nodes producing blocks with same keys
- [ ] Cross-validation of produced blocks
- [ ] Consensus state tracking through epoch transitions
- [ ] VRF output comparison at slot boundaries
- [ ] Fork resolution behavior

**Technical notes:**

- Reference existing VRF comparison code in testing framework
- Consensus state is complex; focus on observable outputs
- Consider slot-by-slot comparison for detailed analysis

**Labels:** `shadow-testing`, `scenarios`, `consensus`, `phase-5`

---

### Issue 5.3: Transaction Processing Scenarios

**Title:** Shadow testing: Implement transaction processing test scenarios

**Description:**

Create scenarios validating that both implementations process various
transaction types identically.

**Acceptance criteria:**

- [ ] Payment transactions (simple transfers)
- [ ] Delegation transactions
- [ ] zkApp commands (various precondition combinations)
- [ ] Fee transfer handling
- [ ] Transaction rejection scenarios (invalid nonce, insufficient balance)

**Technical notes:**

- Generate transactions programmatically for coverage
- Include edge cases (zero amounts, max values)
- Validate transaction pool behavior too

**Labels:** `shadow-testing`, `scenarios`, `transactions`, `phase-5`

---

### Issue 5.4: Network and Edge Case Scenarios

**Title:** Shadow testing: Implement network and edge case test scenarios

**Description:**

Create scenarios for network conditions and protocol edge cases that may expose
implementation differences.

**Acceptance criteria:**

- [ ] Peer discovery and connection behavior
- [ ] Message propagation with delays
- [ ] Network partition and reconnection
- [ ] Slot boundaries and edge cases
- [ ] Epoch transitions
- [ ] Token creation and custom token operations

**Technical notes:**

- Network scenarios may need simulation infrastructure
- Edge cases should be derived from protocol spec
- Document any known behavioral differences

**Labels:** `shadow-testing`, `scenarios`, `edge-cases`, `phase-5`

---

## Phase 6: CI Integration

### Issue 6.1: Shadow Test GitHub Actions Workflow

**Title:** Shadow testing: Integrate shadow tests into CI pipeline

**Description:**

Create GitHub Actions workflows to run shadow tests automatically on PRs and the
develop branch.

**Acceptance criteria:**

- [ ] Workflow triggered on PRs with specific label
- [ ] Quick sanity tests on all PRs (optional)
- [ ] Extended tests on develop branch merges
- [ ] Nightly comprehensive test runs
- [ ] Proper artifact collection on failure

**Technical notes:**

- Reference existing CI patterns in `.github/workflows/`
- Consider test matrix for different scenarios
- Balance thoroughness with CI time

**Labels:** `shadow-testing`, `ci`, `phase-6`

---

### Issue 6.2: Test Result Reporting

**Title:** Shadow testing: Implement test result reporting and dashboard

**Description:**

Create reporting infrastructure to visualize shadow test results and track
comparison status over time.

**Acceptance criteria:**

- [ ] Generate summary reports for CI runs
- [ ] Comparison matrix showing tested features
- [ ] Divergence reports with context
- [ ] Historical trend visualization
- [ ] Integration with GitHub PR comments

**Technical notes:**

- Consider markdown reports for simple integration
- JSON output for external dashboard consumption
- Include links to detailed logs

**Labels:** `shadow-testing`, `ci`, `reporting`, `phase-6`

---

### Issue 6.3: Documentation and Runbook

**Title:** Shadow testing: Create documentation and operational runbook

**Description:**

Document the shadow testing infrastructure, how to run tests locally, and how to
diagnose and resolve divergences.

**Acceptance criteria:**

- [ ] Architecture documentation in `website/docs/developers/`
- [ ] Local development guide (running shadow tests)
- [ ] Troubleshooting guide for common divergences
- [ ] Runbook for CI failures
- [ ] API documentation for testing utilities

**Technical notes:**

- Follow existing documentation patterns
- Include worked examples
- Keep updated as infrastructure evolves

**Labels:** `shadow-testing`, `documentation`, `phase-6`

---

## Summary

| Phase     | Issues | Focus Area                |
| --------- | ------ | ------------------------- |
| 1         | 4      | Foundation infrastructure |
| 2         | 4      | Ledger validation         |
| 3         | 4      | Continuous monitoring     |
| 4         | 4      | Advanced analysis         |
| 5         | 4      | Test scenarios            |
| 6         | 3      | CI integration            |
| **Total** | **23** |                           |

## Dependencies

```
Phase 1 ─┬─> Phase 2 ───> Phase 4
         └─> Phase 3 ───> Phase 4
                           │
Phase 5 (can start after Phase 1)
                           │
         ┌─────────────────┘
         v
      Phase 6
```

- **Phase 1** must complete before Phases 2 and 3
- **Phase 2** and **Phase 3** can proceed in parallel
- **Phase 4** requires both Phase 2 and 3
- **Phase 5** can start after Phase 1 (scenarios use foundation)
- **Phase 6** should be last (integrates all other work)

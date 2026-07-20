# OpenMina Fuzzing Infrastructure

**Note: This document is very incomplete and contains unverified claims.**

This document explains the fuzzing infrastructure for testing OpenMina's
transaction processing logic against the reference OCaml implementation.

## Overview

The OpenMina fuzzer is a differential testing system that validates the Rust
implementation of the Mina Protocol by comparing it against the OCaml reference
implementation. It focuses on transaction processing, validation, and ledger
state management.

## Architecture

### Components

The fuzzer is located in `tools/fuzzing/` and implements differential fuzzing
between the OCaml and Rust implementations of some components.

## What Gets Fuzzed

### Primary Targets

**Transaction Pool Validation** (`pool_verify`)

- Validates transactions before adding to mempool

**Transaction Application** (`apply_transaction`)

- Applies transactions to ledger state

## Mutation Strategies

The fuzzer uses mutation strategies implemented in
`tools/fuzzing/src/transaction_fuzzer/mutator.rs`:

**Weighted Random Selection:**

- Uses `rand_elements()` function that gives more weight to fewer mutations
- Comment in code: "We give more weight to smaller amount of elements since in
  general we want to perform fewer mutations"

## Running the Fuzzer

### Prerequisites

**Rust Nightly Toolchain:**

```bash
rustup toolchain install nightly
rustup override set nightly
```

**OCaml Reference Implementation:**

The OCaml fuzzer loop is implemented in
[transaction_fuzzer.ml](https://github.com/openmina/mina/blob/openmina/fuzzer/src/app/transaction_fuzzer/transaction_fuzzer.ml).

```bash
# Use the openmina/fuzzer branch from https://github.com/openmina/mina
# Branch: openmina/fuzzer
# Build the transaction fuzzer executable in that branch:
# dune build src/app/transaction_fuzzer/transaction_fuzzer.exe

# Then set the path to the built executable:
export OCAML_TRANSACTION_FUZZER_PATH=/path/to/mina/_build/default/src/app/transaction_fuzzer/transaction_fuzzer.exe
```

**Note**: The `openmina/fuzzer` branch is messy and should be cleaned up and
integrated into mina mainline to ease the process.

### Basic Usage

**Default Fuzzing:**

```bash
cd tools/fuzzing
cargo run --release
```

**With Specific Configuration:**

```bash
# Use specific random seed for reproducibility
cargo run --release -- --seed 12345

# Enable specific fuzzing modes
cargo run --release -- --pool-fuzzing true --transaction-application-fuzzing true

# Reproduce a specific failing case
cargo run --release -- --fuzzcase /path/to/fuzzcase.file
```

### Configuration Options

**Command Line Arguments:**

- `--seed <NUMBER>` - Set random seed for reproducible runs
- `--pool-fuzzing <BOOL>` - Enable/disable pool validation fuzzing
- `--transaction-application-fuzzing <BOOL>` - Enable/disable transaction
  application fuzzing
- `--fuzzcase <PATH>` - Reproduce specific failing test case

**Environment Variables:**

- `OCAML_TRANSACTION_FUZZER_PATH` - Path to OCaml transaction fuzzer executable
- `FUZZCASES_PATH` - Directory to save failing cases (default: `/tmp/`)

### Internal Configuration

**Verified Parameters (from main.rs):**

- **Initial Accounts:** 1000 accounts created at startup
- **Minimum Fee:** 1,000,000 currency units (default in context.rs)
- **Default Seed:** 42
- **Coverage Updates:** Every 1000 iterations
- **Snapshots:** Every 10000 iterations

**Additional Configuration:** See
`tools/fuzzing/src/transaction_fuzzer/context.rs` for cache sizes and other
parameters.

## Coverage Analysis

The fuzzer includes coverage tracking using LLVM instrumentation.

**Usage:**

```bash
# Coverage collection is built into the fuzzer when using nightly toolchain
# The fuzzer automatically tracks coverage and generates reports
# See coverage implementation in main.rs CoverageStats struct
cargo run --release
```

## Technical Implementation

### Binary Protocol Communication

**OCaml Interoperability:**

- Uses `binprot` serialization for data exchange
- Implements length-prefixed message framing
- Command-based interaction model with stdin/stdout communication

**Communication Protocol:** The OCaml fuzzer supports multiple action types:

- `SetConstraintConstants` - Configure blockchain constraint parameters
- `InitializeAccounts` - Setup initial account states
- `SetupTransactionPool` - Initialize transaction pool
- `VerifyPoolTransaction` - Validate transactions for pool admission
- `ApplyTransaction` - Apply transactions to ledger state
- `GetAccounts` - Retrieve account information
- `Exit` - Terminate fuzzer process

**Rust Integration:** See `main.rs` functions:

- `ocaml_pool_verify()` - Pool validation testing
- `ocaml_apply_transaction()` - Transaction application testing
- `serialize()`/`deserialize()` - Binary protocol communication

### OCaml Fuzzer Architecture

**Core Components:**

- **Ledger Simulation**: Creates ephemeral ledgers for isolated testing
- **Mock Components**: Uses `Mock_transition_frontier` for controlled blockchain
  simulation
- **Async Operations**: Leverages OCaml's Async library for non-blocking
  operations
- **Error Tracking**: Comprehensive error handling with backtrace generation

**Testing Environment:**

- Simulated blockchain ledger with configurable constraint constants
- Account initialization and management
- Transaction pool setup and verification
- Isolated transaction application testing

**Communication Model:**

- Loop-based command processing from stdin
- Binary-encoded responses to stdout
- Supports graceful termination via `Exit` command

### Error Handling and Debugging

**Panic Detection:** Implemented in `main.rs` `fuzz()` function using
`panic::catch_unwind()` to detect panics and save fuzzcases for reproduction.

**OCaml Error Handling:**

- Comprehensive error tracking with backtrace generation
- Structured error responses for debugging
- Async error propagation for non-blocking operations

## Basic Usage Guide

1. **Reproduce with Fixed Seed** - Use `--seed` to reproduce specific runs
2. **Examine Saved Cases** - Check `/tmp/` for automatically saved failing cases
3. **Check OCaml Connectivity** - Ensure OCaml fuzzer path is correct

## Troubleshooting

**OCaml Process Communication Failures:**

```bash
# Check OCaml fuzzer path
ls -la $OCAML_TRANSACTION_FUZZER_PATH

# Test OCaml fuzzer directly
$OCAML_TRANSACTION_FUZZER_PATH --help
```

**Coverage Collection Problems:**

```bash
# Ensure nightly toolchain
rustup show

# Coverage is built into the fuzzer, no additional tools needed
```

**Permission Denied Errors:**

```bash
# Check write permissions for fuzzcase directory
ls -la /tmp/

# Use alternative directory
export FUZZCASES_PATH=/path/to/writable/directory
```

# Poseidon Migration Plan

This document outlines the strategy for removing the local `poseidon` crate and eliminating duplicated Poseidon logic within the `mina-rust` workspace, specifically in `crates/ledger`. All implementations will be consolidated to use the `mina-poseidon` crate from the `proof-systems` repository.

## 1. Objectives
- Remove the local `poseidon` crate at the root of the workspace.
- Eliminate duplicated Poseidon logic in `crates/ledger/src/proofs/transaction.rs` and `crates/ledger/src/proofs/opt_sponge.rs`.
- Standardize on `mina-poseidon` for both protocol-level (P2P/Node) and circuit-level (Ledger/SNARK) code.
- Ensure cryptographic consistency across all replacements.

## 2. Background & Constraints
The local `poseidon` crate is currently used for its simplicity in non-circuit contexts. However, `crates/ledger` duplicates this logic because it needs to inject `Witness<F>::exists` calls into the sponge operations to record the witness trace for SNARK proof generation.

`mina-poseidon` is the authoritative source for:
- `PlonkSpongeConstantsKimchi`
- `PlonkSpongeConstantsLegacy`
- MDS Matrices and Round Constants for `Fp` and `Fq` (Pasta curves).

## 3. Migration Phases

### Phase 1: API Discovery & Mapping
- **Action**: Inspect `mina-poseidon` (via `cargo doc` or checking the dependency) to identify the equivalent types for:
    - `SpongeConstants` trait and its implementors.
    - `SpongeParams` (usually `ArithmeticSpongeParams`).
    - `ArithmeticSponge` / `FqSponge`.
- **Validation**: Ensure `mina-poseidon` exposes the necessary internal functions (like `sbox` or `apply_mds_matrix`) or allow a custom "Witness-aware" implementation to use its constants.

### Phase 2: Migrate "Pure" Contexts
Replace usage of the root `poseidon` crate in crates that do not require SNARK witness generation.
- **Affected Crates**:
    - `crates/p2p-messages`
    - `crates/vrf`
    - `crates/node` (various sub-crates)
    - `tools/hash-tool`
- **Action**: Update `Cargo.toml` to remove `poseidon` and ensure `mina-poseidon` is used. Update imports in `.rs` files.

### Phase 3: Refactor `crates/ledger` (Witness-Aware Logic)
The `ledger` crate requires specialized sponge logic for witness generation.
- **`crates/ledger/src/proofs/transaction.rs`**:
    - Remove the internal `mod poseidon` and `Sponge` implementation.
    - If `mina-poseidon` supports a generic "Witness" recorder, use it.
    - Otherwise, create a wrapper struct that uses `mina-poseidon` constants and matrices but maintains the `w.exists()` calls required by the `ledger` prover logic.
- **`crates/ledger/src/proofs/opt_sponge.rs`**:
    - Refactor to use constants from `mina-poseidon`.
    - Ensure `block_cipher` and `full_round` logic matches the `mina-poseidon` specification exactly.

### Phase 4: Verification
- **Unit Tests**: Run `cargo test -p mina-tree` and `cargo test -p mina-poseidon` (if applicable) to ensure no regressions in hashing.
- **Integration Tests**: Execute ledger-specific proof generation tests to verify that the witness trace remains correct and proofs are still valid.
- **Comparison**: Use the `hash-tool` to compare results between the new implementation and the OCaml reference if possible.

### Phase 5: Cleanup
- Remove `poseidon` member from the workspace `Cargo.toml`.
- Delete the `poseidon/` directory.
- Audit all remaining `Cargo.toml` files to ensure no references to the local `poseidon` crate remain.

## 4. Specific Symbol Mapping (Tentative)

| Local `poseidon` Symbol | `mina-poseidon` Equivalent |
| :--- | :--- |
| `SpongeConstants` | `kimchi::circuits::scalars::SpongeConstants` (or similar) |
| `PlonkSpongeConstantsKimchi` | `mina_poseidon::constants::PlonkSpongeConstantsKimchi` |
| `PlonkSpongeConstantsLegacy` | `mina_poseidon::constants::PlonkSpongeConstantsLegacy` |
| `Sponge` | `mina_poseidon::sponge::ArithmeticSponge` |
| `SpongeParams` | `mina_poseidon::sponge::ArithmeticSpongeParams` |

## 5. Success Criteria
1. The project builds successfully without the `poseidon` crate.
2. All `ledger` tests (especially proof generation) pass.
3. `cargo doc` shows `mina-tree` (ledger) linking to `mina-poseidon` types.

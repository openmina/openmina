# Poseidon Migration Plan

## Goal
Remove the local `poseidon` crate and standardize all hashing (Pure and Witness-Aware) on `mina-hasher` and `mina-poseidon` from `proof-systems`.

## Phase 1: Pure Hashing (Non-Witness)
**Target:** `crates/vrf`, `crates/ledger` (Verification Logic)

- [x] **`mina-vrf` Migration:**
    - [x] Replace `poseidon` with `mina-hasher`.
    - [x] Fix Big-Endian packing in `to_roinput`.
    - [x] Verify output with tests (Seed, Output, Threshold).
- [x] **`mina-tree` Verification Logic:**
    - [x] Replace `poseidon` in `verifier/common.rs`.
    - [x] Implement `LegacyInputs` helper for legacy packing.
    - [x] Fix `TransactionUnionPayload` hashing.
    - [x] Verify with `cargo nextest`.

## Phase 2: Domain Separation & Constants
**Target:** Global

- [x] **Domain Strings:**
    - [x] Map `mina_core::NetworkConfig` to `mina_signer::NetworkId`.
    - [x] Ensure `mina_hasher` uses correct domain bytes.
- [x] **Constants:**
    - [x] `mina-vrf` uses correct generator points (Testnet/Mainnet).
    - [x] `mina-tree` uses correct `Vesta` params.

## Phase 3: Witness-Aware Logic (Circuit Generation)
**Target:** `crates/ledger/src/proofs/` (`transaction.rs`, `opt_sponge.rs`, `wrap.rs`)

- [x] **Refactor `transaction.rs`:**
    - [x] Replace `poseidon::Sponge` with local `Sponge` using `mina_poseidon` structure.
    - [x] Create `proofs/poseidon_params.rs` (copy of `poseidon/src/params.rs`) to supply constants.
    - [x] Implement `SpongeParamsForField` to bridge local `Sponge` and constants.
    - [x] Replace `legacy::Inputs` with local `LegacyInputs` struct.
- [x] **Refactor `opt_sponge.rs` & `wrap.rs`:**
    - [x] Update `SpongeState` usage to `mina_poseidon::poseidon::SpongeState`.
    - [x] Fix trait bounds (`SpongeParamsForField`) for `F`.
    - [x] Fix dereference logic in `apply_mds_matrix`.
- [x] **Verification:**
    - [x] Compiles successfully.
    - [x] `cargo test -p mina-tree` passes (149 tests).
    - [x] `test_transaction_logic` passes.

## Phase 4: Final Cleanup

**Target:** Workspace



- [x] **Remove Residual Imports:**

    - [x] Remove `use ::poseidon::...` in `transaction.rs` (`LazyParam`, `MINA_PROTO_STATE_BODY`).

    - [x] Replace `LazyParam` with `CustomDomain` newtype and runtime domain strings.

- [x] **Standardize Mutable Interface:**

    - [x] Implemented `Inputs` wrapper around `ROInput` in `hash.rs` to maintain mutable API and minimize codebase-wide changes.

- [x] **Remove Crate:**

    - [x] Remove `poseidon` from all `Cargo.toml` files.

    - [x] Remove `poseidon` from `Cargo.toml` (workspace).

    - [x] Delete `poseidon` directory.

- [ ] **Final Verification:**

    - [ ] `mina-p2p-messages` fixed and compiling.

    - [ ] `mina-core` fixed and compiling.

    - [x] `mina-tree` fixed and compiling.

    - [ ] `mina-snark` pending fixes for `MerkleTreeNode` and `DomainParameter`.

    - [ ] Full workspace build and test.

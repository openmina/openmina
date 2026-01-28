# Poseidon Migration Execution Plan

## Overview

Migrate all code from the local `poseidon` crate to `mina_hasher` (from proof-systems).

### Migration Pattern

The local `poseidon` crate provides:
- `Inputs` - helper for packing fields/bits
- `LazyParam` - pre-computed sponge states with domain separators
- `hash_with_kimchi(param, fields)` - hash with domain separator
- Various pre-computed params (MINA_ACCOUNT, merkle tree heights, etc.)

The `mina_hasher` crate provides:
- `ROInput` - equivalent to `Inputs`
- `Hashable` trait with `domain_string()` for domain separation
- `create_kimchi::<H>()` to create a hasher

**Standard migration pattern:**
```rust
// Before (poseidon)
use poseidon::hash::{hash_with_kimchi, params::SOME_PARAM, Inputs};
let mut inputs = Inputs::new();
inputs.append_field(x);
let hash = hash_with_kimchi(&SOME_PARAM, &inputs.to_fields());

// After (mina_hasher)
use mina_hasher::{Hashable, Hasher, ROInput};

#[derive(Clone)]
struct SomeHashable(ROInput);
impl Hashable for SomeHashable {
    type D = ();
    fn to_roinput(&self) -> ROInput { self.0.clone() }
    fn domain_string(_: ()) -> Option<String> { Some("SomeDomain".to_string()) }
}

let inputs = ROInput::new().append_field(x);
let hash = mina_hasher::create_kimchi::<SomeHashable>(())
    .update(&SomeHashable(inputs))
    .digest();
```

---

## Phase 1: p2p-messages Crate

### 1.1 hash_input.rs
- Replace `use poseidon::hash::Inputs` with `use mina_hasher::ROInput`
- Change `FailableToInputs` trait to use `ROInput` instead of `Inputs`
- Update all `impl FailableToInputs` methods

### 1.2 v2/manual.rs
- Replace `NO_INPUT_COINBASE_STACK` usage in `CoinbaseStackData::empty()`
- Use `mina_hasher::create_kimchi` with domain "CoinbaseStack"

### 1.3 v2/hashing.rs
- Replace `hash_with_kimchi` and `Inputs` usage
- Create domain-specific `Hashable` wrappers for protocol state hashing
- Update `MinaHash` implementations

### 1.4 Cargo.toml
- Remove `poseidon = { workspace = true }`
- Add `mina-hasher = { workspace = true }`

---

## Phase 2: core Crate

### 2.1 network.rs
**Challenge:** `NetworkConfig` stores `&'static poseidon::hash::LazyParam` references.

**Solution:** Create equivalent static `&'static str` domain strings or pre-compute
the hash values needed. The `LazyParam` is only used to provide pre-absorbed
domain separator state - we can achieve the same with `domain_string()`.

- Change `signature_prefix`, `legacy_signature_prefix`, `account_update_hash_param`
  from `&'static LazyParam` to `&'static str` (the domain string)
- Update callers to use `mina_hasher` with these domain strings

### 2.2 Cargo.toml
- Remove `poseidon = { workspace = true }`
- Add `mina-hasher = { workspace = true }` if needed

---

## Phase 3: ledger Crate (Most Complex)

This crate has the most extensive poseidon usage. Key files:

### 3.1 Simple replacements (hash_with_kimchi -> mina_hasher)
- `hash.rs` - main hash utilities
- `account/account.rs` - account hashing
- `common.rs` - signature verification hashing
- `verifier/common.rs` - similar to common.rs
- `staged_ledger/hash.rs` - staged ledger hashing
- `proofs/block.rs` - VRF and epoch seed hashing
- `proofs/zkapp.rs` - zkapp hashing
- `zkapps/snark.rs` - zkapp snark hashing

### 3.2 Merkle tree params (special handling)
- `tree_version.rs` - uses `get_merkle_param_for_height`
- `sparse_ledger/sparse_ledger_impl.rs` - same

**Solution:** Create a helper that generates domain strings like "MinaMklTree000"
through "MinaMklTree035" dynamically.

### 3.3 Transaction logic (most complex)
- `scan_state/transaction_logic.rs`
- `scan_state/transaction_logic/*.rs` (multiple submodules)
- `scan_state/pending_coinbase.rs`
- `scan_state/protocol_state.rs`
- `scan_state/fee_excess.rs`
- `scan_state/scan_state.rs`

### 3.4 Proof-related (may need witness-aware handling)
- `proofs/field.rs` - uses `SpongeParamsForField`
- `proofs/transaction.rs` - internal poseidon sponge for witness generation
- `proofs/public_input/*.rs`

**Note:** The `proofs/transaction.rs` file has its own internal `mod poseidon`
which implements witness-aware sponge operations. This may need to remain
or be refactored to use `mina_poseidon` constants.

### 3.5 Cargo.toml
- Remove `poseidon = { workspace = true }`
- Keep `mina-poseidon = { workspace = true }` (already present)
- Add `mina-hasher = { workspace = true }`

---

## Phase 4: tools/fuzzing Crate

- Check usage and apply same patterns
- Remove `poseidon` dependency

---

## Phase 5: Cleanup

### 5.1 Remove poseidon from workspace
- Remove from `Cargo.toml` members list
- Remove from workspace dependencies
- Delete `poseidon/` directory

### 5.2 Verify
- `cargo check --workspace`
- `cargo test -p mina-tree` (ledger tests)
- `cargo test -p mina-vrf`

---

## Estimated Scope

| Crate | Files | Complexity |
|-------|-------|------------|
| p2p-messages | 3 | Low |
| core | 1 | Medium |
| ledger | ~20 | High |
| tools/fuzzing | 1 | Low |

---

## Questions Before Proceeding

1. Should `proofs/transaction.rs` internal poseidon module be migrated or kept
   (it has witness-aware logic)?
2. For `core/network.rs`, is changing the type from `&'static LazyParam` to
   `&'static str` acceptable? This is a breaking API change.

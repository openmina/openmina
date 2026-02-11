# Test Failure Summary (Poseidon PR)

Run: `cargo nextest run --workspace --no-fail-fast --lib`
Date: 2026-02-11
Branch: `rp/poseidon-arrays`

---

## CI Jobs That Run Tests

| CI Job | Command | Crate(s) |
|--------|---------|----------|
| `unit-tests` | `cargo nextest run --workspace --lib` (with exclusions) | mina-tree, mina-vrf, redux, alloc-test, xtask |
| `ledger-tests` | `cargo +nightly test --release` in `crates/ledger/` | mina-tree (all test types) |
| `mina-node-native-tests` | `cargo test -p mina-node-native --all-features --release --tests` | mina-node-native |
| `p2p-messages-tests` | `cargo test -p mina-p2p-messages --tests --release` | mina-p2p-messages |
| `p2p-tests` | `cargo test -p mina-p2p --tests --release` | mina-p2p |
| `account-tests` | `cargo test -p mina-node-account` | mina-node-account |
| `scenario-tests` | Pre-built binaries from mina-node-testing | Integration tests |
| `wallet-tests` | Shell scripts | E2E tests |

---

## CI Failures — mina-tree (53 failures)

Tested by: `unit-tests` job + `ledger-tests` job

**Dominant root cause (51/53):**
`assertion failed: prefix.len() <= MAX_DOMAIN_STRING_LEN`
at `proof-systems/hasher/src/lib.rs:176`

The `HashParam` enum's `.to_string()` is producing domain strings that
exceed `mina_hasher`'s `MAX_DOMAIN_STRING_LEN` (20 bytes).

### account tests (4)
- [ ] `account::account::tests::test_hash_account`
- [ ] `account::account::tests::test_hash_genesis_winner_account`
- [ ] `account::account::tests::test_rand`
- [ ] `account::account::tests::test_dummy_sideloaded_verification_key`
      (different cause: panics at `account.rs:1967`)

### database tests (9)
- [ ] `database::database::tests::test_hash_empty`
- [ ] `database::database::tests::test_hashing_tree`
- [ ] `database::database::tests::test_root_hash_different_orders`
- [ ] `database::database::tests_ocaml::test_create_empty_doesnt_modify_hash`
- [ ] `database::database::tests_ocaml::test_get_set_all_same_root_hash`
- [ ] `database::database::tests_ocaml::test_merkle_path_long`
- [ ] `database::database::tests_ocaml::test_merkle_path_test2`
- [ ] `database::database::tests_ocaml::test_remove_restore_root_hash`
- [ ] `database::database::tests_ocaml::test_set_batch_accounts_change_root_hash`

### mask tests (14)
- [ ] `mask::mask::tests::test_cached_merkle_path`
- [ ] `mask::mask::tests::test_masks`
- [ ] `mask::mask::tests::test_masks_cached_hashes`
- [ ] `mask::mask::tests::test_merkle_path_one_account`
- [ ] `mask::mask::tests_ocaml::test_agree_on_root_hash_after_set`
- [ ] `mask::mask::tests_ocaml::test_agree_on_root_hash_before_set`
- [ ] `mask::mask::tests_ocaml::test_create_empty_doesnt_modify_the_hash`
- [ ] `mask::mask::tests_ocaml::test_mask_and_parent_agree_on_merkle_path`
- [ ] `mask::mask::tests_ocaml::test_parent_mask_agree_on_hashes`
- [ ] `mask::mask::tests_ocaml::test_parent_mask_agree_on_hashes_set_parent_only`
- [ ] `mask::mask::tests_ocaml::test_removing_accounts_from_mask_restore_root_hash`
- [ ] `mask::mask::tests_ocaml::test_removing_accounts_from_parent_and_mask_restore_root_hash`
- [ ] `mask::mask::tests_ocaml::test_removing_accounts_from_parent_restore_root_hash`
- [ ] `mask::mask::tests_ocaml::test_validate_inner_hashes`

### scan_state tests (1)
- [ ] `scan_state::pending_coinbase::tests::test_merkle_tree`
      (different cause: panics at `pending_coinbase.rs:1265`)

### staged_ledger tests (25)
- [ ] `staged_ledger::tests_ocaml::be_able_to_include_random_number_of_commands_many_failed`
- [ ] `staged_ledger::tests_ocaml::be_able_to_include_random_number_of_commands_many_normal`
- [ ] `staged_ledger::tests_ocaml::be_able_to_include_random_number_of_commands_one_prover_failed`
- [ ] `staged_ledger::tests_ocaml::be_able_to_include_random_number_of_commands_one_prover_normal`
- [ ] `staged_ledger::tests_ocaml::blocks_having_commands_with_sufficient_funds_are_rejected`
- [ ] `staged_ledger::tests_ocaml::check_zero_fee_excess_for_partitions`
- [ ] `staged_ledger::tests_ocaml::commands_with_insufficient_funds_are_not_included`
- [ ] `staged_ledger::tests_ocaml::de_serialize_zkapps`
- [ ] `staged_ledger::tests_ocaml::max_throughput_ledger_proof_count_fixed_blocks`
- [ ] `staged_ledger::tests_ocaml::max_throughput_ledger_proof_count_fixed_blocks_one_prover`
- [ ] `staged_ledger::tests_ocaml::max_throughput_normal`
- [ ] `staged_ledger::tests_ocaml::max_throughput_normal_one_prover`
- [ ] `staged_ledger::tests_ocaml::max_throughput_random_fee`
- [ ] `staged_ledger::tests_ocaml::max_throughput_random_number_fee_number_of_proofs_worst_case_provers`
- [ ] `staged_ledger::tests_ocaml::max_throughput_random_number_of_proofs_worst_case_provers`
- [ ] `staged_ledger::tests_ocaml::provers_cant_pay_the_account_creation_fee`
- [ ] `staged_ledger::tests_ocaml::random_number_of_commands_random_number_of_proofs_one_prover`
- [ ] `staged_ledger::tests_ocaml::random_number_of_transactions_random_number_of_proofs_worst_case_provers`
- [ ] `staged_ledger::tests_ocaml::supercharged_coinbase_locked_account_delegating_to_locked_account`
- [ ] `staged_ledger::tests_ocaml::supercharged_coinbase_locked_account_delegating_to_unlocked_account`
- [ ] `staged_ledger::tests_ocaml::supercharged_coinbase_staking`
- [ ] `staged_ledger::tests_ocaml::supercharged_coinbase_unlocked_account_delegating_to_locked_account`
- [ ] `staged_ledger::tests_ocaml::validate_pending_coinbase_for_random_number_of_commands_many_prover`
- [ ] `staged_ledger::tests_ocaml::validate_pending_coinbase_for_random_number_of_commands_one_prover`
- [ ] `staged_ledger::tests_ocaml::zero_proof_fee_should_not_create_a_fee_transfer`

---

## CI Failures — mina-vrf (2 tests, outcome unknown)

Tested by: `unit-tests` job

These two tests were still running after 540s in debug mode when we killed
the run. They may pass in release mode (which is how `ledger-tests` runs).

- `tests::test_first_winning_slot`
- `tests::test_slot_calculation_time_big_producer`

---

## Root Cause Summary

| Priority | Root Cause | Tests Affected |
|----------|-----------|---------------|
| 1 | `prefix.len() > MAX_DOMAIN_STRING_LEN` — HashParam Display too long | 51 |
| 2 | `test_dummy_sideloaded_verification_key` (account.rs:1967) | 1 |
| 3 | `test_merkle_tree` (pending_coinbase.rs:1265) | 1 |
| 4 | mina-vrf slow tests (unknown outcome) | 2 |

**Fix priority 1 first** — it's the single root cause behind 51 of
the 53 CI failures.

---

## Root Cause Analysis: Domain String Comparison (Rust vs OCaml)

### OCaml source of truth

`hash_prefixes.ml` defines `length_in_bytes = 20`. Strings shorter than
20 chars are right-padded with `*`. Strings longer than 20 are truncated.
The `MAX_DOMAIN_STRING_LEN` assert in proof-systems (`hasher/src/lib.rs`)
enforces this same 20-byte limit.

### All shared domain strings match exactly

Every `HashParam` variant that corresponds to an OCaml `hash_prefixes`
constant uses the identical string and is <= 20 chars. No discrepancies.

### The offenders are Rust-only inventions

These domain strings do NOT exist in OCaml's `hash_prefixes.ml`:

| HashParam variant | String | Length | Assertion |
|---|---|---|---|
| `ZkappActionStateEmptyElt` | `MinaZkappActionStateEmptyElt` | **28** | FAILS (>20) |
| `ZkappActionsEmpty` | `MinaZkappActionsEmpty` | **21** | FAILS (>20) |
| `ZkappEventsEmpty` | `MinaZkappEventsEmpty` | 20 | Passes (==20) |
| `NoInputCoinbaseStack` | `CoinbaseStack` | 13 | Passes (ok) |

The first two trigger the assertion that causes 51 test failures. All
tests that hash an account hit `ZkappActionStateEmptyElt` because every
account contains an action state field whose empty value is computed via
`hash_noinputs(HashParam::ZkappActionStateEmptyElt)`.

### How OCaml computes these "empty" hashes

OCaml does NOT use separate domain strings for empty values. It uses the
**standard** domain string and squeezes with no additional input:

- Empty action state element: `salt("MinaZkappSeqEvents") |> squeeze`
- Empty events: `salt("MinaZkappEvents") |> squeeze`
- Empty actions: `salt("MinaZkappSeqEvents") |> squeeze`
- Empty coinbase stack: `salt("CoinbaseStack") |> squeeze`

The Rust `hash_noinputs()` (`crates/ledger/src/hash.rs:102`) does
the same operation — creates a hasher with a domain, calls `.digest()`
with nothing absorbed — but uses wrong domain strings for 3 of 4 cases.

---

## Proposed Fix

Replace the invented domain strings with the standard ones that OCaml
uses. The `hash_noinputs` operation is correct; only the domain param
passed to it is wrong.

### Changes to `crates/core/src/hash_param.rs`

Delete these enum variants and their string mappings:
- `ZkappActionStateEmptyElt` (mapped to `"MinaZkappActionStateEmptyElt"`)
- `ZkappEventsEmpty` (mapped to `"MinaZkappEventsEmpty"`)
- `ZkappActionsEmpty` (mapped to `"MinaZkappActionsEmpty"`)

`NoInputCoinbaseStack` already maps to `"CoinbaseStack"` (same as
`CoinbaseStack`), so it could also be removed as a redundant alias.

### Changes to call sites

| File | Line | Before | After |
|------|------|--------|-------|
| `crates/ledger/src/account/account.rs` | 891 | `hash_noinputs(HashParam::ZkappActionStateEmptyElt)` | `hash_noinputs(HashParam::ZkappSeqEvents)` |
| `crates/ledger/src/scan_state/transaction_logic/zkapp_command.rs` | 99 | `HashParam::ZkappEventsEmpty` | `HashParam::ZkappEvents` |
| `crates/ledger/src/scan_state/transaction_logic/zkapp_command.rs` | 120 | `HashParam::ZkappActionsEmpty` | `HashParam::ZkappSeqEvents` |
| `crates/ledger/src/scan_state/pending_coinbase.rs` | 177 | `HashParam::NoInputCoinbaseStack` | `HashParam::CoinbaseStack` |

# Poseidon Domain String Analysis & Fix Plan

## Context

PR #2134 introduces Poseidon hashing changes. 51 of 53 CI test failures
stem from a `prefix.len() <= MAX_DOMAIN_STRING_LEN` assertion in the
proof-systems `mina_hasher` crate. The OCaml codebase uses domain strings
longer than 20 characters (e.g. `"MinaZkappActionStateEmptyElt"` at 28
chars). The Rust proof-systems `mina_hasher` enforces a 20-character
limit. We need to match OCaml's hashing behavior exactly.

---

## Root Cause Analysis

### OCaml has two distinct domain string mechanisms

**Mechanism 1: `Hash_prefixes.create` (20-byte star-padded)**
- File: `mina/src/lib/hash_prefixes/hash_prefixes.ml`
- Truncates strings > 20 bytes, pads shorter strings with `'*'` to
  exactly 20 bytes
- Used for predefined hash prefix constants ("MinaAccount",
  "CoinbaseStack", etc.)
- These flow through `Hash_prefix_states` which calls
  `Random_oracle.salt(star_padded_string)` to create sponge initial
  states used as `~init` for `Random_oracle.hash ~init:state fields`

**Mechanism 2: Direct `Hash_prefix_create.salt` (up to 31 chars, NO
padding)**
- File: `mina/src/lib/random_oracle/random_oracle.ml:95-100`
- `prefix_to_field(s)`: asserts `8 * len < 255` (allows up to 31 chars),
  converts string bits to a field element via `Field.project`
- `salt(s)`: absorbs `prefix_to_field(s)` into initial sponge state
- Used in `zkapp_account.ml` for "empty" hashes:
  - `"MinaZkappActionStateEmptyElt"` (28 chars) - empty action state
  - `"MinaZkappActionsEmpty"` (21 chars) - empty actions hash
  - `"MinaZkappEventsEmpty"` (20 chars) - empty events hash
  - `"CoinbaseStack"` (13 chars) - empty coinbase stack (same string as
    the Hash_prefix, but used WITHOUT star-padding)

### How they differ

For a string like `"CoinbaseStack"` (13 chars):

| Path | Processing | Field element from |
|------|-----------|-------------------|
| Hash_prefixes.create | star-pad to `"CoinbaseStack*******"` (20 bytes) | `prefix_to_field("CoinbaseStack*******")` |
| Hash_prefix_create.salt | use raw string | `prefix_to_field("CoinbaseStack")` |

These produce **different** field elements. Both are correct — they're
used in different contexts.

### Rust proof-systems only implements Mechanism 1

- File: `proof-systems/hasher/src/lib.rs:169-183`
  (`domain_prefix_to_field`)
- Enforces `prefix.len() <= 20`, star-pads to 20 chars, zero-extends
  to field size, converts via `F::from_bytes`
- Called during `create_kimchi`/`create_legacy` hasher initialization
- There is no Rust equivalent of OCaml's more permissive
  `prefix_to_field`

### Why the old `hash_noinputs` was wrong (TWO bugs)

The committed `hash_noinputs` was:
```rust
mina_hasher::create_kimchi::<GenericHashable>(
    CustomDomain(s.to_string())
).digest()
```

**Bug 1 — Star-padding when it shouldn't:**
`create_kimchi` routes through `domain_prefix_to_field` which star-pads
the string. OCaml's `Hash_prefix_create.salt` does NOT star-pad. This
means `hash_noinputs("CoinbaseStack")` produced a field element from
`"CoinbaseStack*******"` instead of from `"CoinbaseStack"`.

**Bug 2 — Returns wrong sponge element:**
The proof-systems hasher's `init()` (poseidon.rs:86-104) does:
1. `absorb([domain_field])` — state = [field, 0, 0], Absorbed(1)
2. `squeeze()` — permutes, returns state[0], advances to Squeezed(1)
3. Saves state

Then `digest()` calls `squeeze()`:
4. Squeezed(1), n=1, rate=2, so returns `state[1]` (second element!)

OCaml's `salt(s) |> digest` returns `state[0]`:
1. `update ~state:initial [| prefix_to_field(s) |]`
   — `to_blocks` (sponge.ml:222-231) pads to a full block [field, 0]
   — `sponge` (sponge.ml:217-219) adds block to state then permutes
   — returns the full permuted state
2. `digest(state)` = `state.(0)` (sponge.ml:250)

### OCaml sponge behavior — key detail

File: `snarky/sponge/sponge.ml:208-256` (`Make_hash` functor)

```ocaml
let sponge perm blocks ~state =
    Array.fold ~init:state blocks ~f:(fun state block ->
        add_block ~state block ; perm state )

let to_blocks rate field_elems =
    let n = Array.length field_elems in
    let num_blocks = if n = 0 then 1 else (n + rate - 1) / rate in
    ...
    Array.init num_blocks ~f:create_block

let update params ~state inputs =
    let state = copy state in
    sponge (block_cipher params) (to_blocks rate inputs) ~state

let digest state = state.(0)
```

Critical: `to_blocks rate [| x |]` with rate=2 produces ONE block
`[| x; 0 |]` (zero-padded to full rate). The `sponge` function adds
this block to state then applies `block_cipher` (Poseidon permutation).
So even for a single input element, the permutation IS applied.

The Rust sponge (poseidon.rs:101-140) behaves differently:
- `absorb([x])`: adds x to state[0], sets Absorbed(1), no permutation
  (rate not full)
- `squeeze()`: sees Absorbed state, calls `poseidon_block_cipher()`,
  returns state[0]

Both end up permuting `[x, 0, 0]` and returning state[0], but they
arrive there by different paths. The OCaml path pads the input to a
full block first; the Rust path doesn't pad but the squeeze forces the
permutation anyway.

### The working tree fix

The new `hash_noinputs` (in working tree, `crates/ledger/src/hash.rs`):
```rust
let param_bytes = s.as_bytes();
let mut bytes = [0u8; 32];
bytes[..param_bytes.len()].copy_from_slice(param_bytes);
let domain_field = Fp::from_random_bytes(&bytes)
    .expect("invalid domain bytes");

let mut sponge = ArithmeticSponge::<
    Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS
>::new(Fp::static_params());
sponge.absorb(&[domain_field]);
sponge.squeeze()
```

This correctly:
1. Converts raw string to field via zero-padded bytes (no star-padding)
2. Absorbs one element, then squeezes — applies one permutation and
   returns `state[0]`

### Byte-to-field conversion equivalence

OCaml `prefix_to_field` (`random_oracle.ml:95-98`):
- `string_bits(s)` extracts bits LSB-first per byte (confirmed in
  `snarky/sponge/fold_lib/fold.ml:105-110`: uses `n lsr i` which is
  logical shift right, extracting bit 0, bit 1, etc.)
- `Field.project(bits)` = `sum(bit_i * 2^i)` = little-endian integer

Rust `Fp::from_random_bytes(bytes)`:
- Interprets bytes as a little-endian integer

Both produce identical field elements for the same input string.

**VERIFIED:** `Fp::from_random_bytes` on `"CoinbaseStack"` zero-padded
to 32 bytes produces:
`436f696e62617365537461636b00000000000000000000000000000000000000`
which is exactly the raw LE byte representation of the ASCII string.

---

## Working Tree Changes Summary

| File | Change |
|------|--------|
| `crates/core/src/hash_param.rs` | Added 4 enum variants for salt-only domain strings |
| `crates/ledger/src/hash.rs` | Rewrote `hash_noinputs` to bypass `domain_prefix_to_field` |
| `crates/ledger/src/account/account.rs:891` | `ZkappSeqEvents` → `ZkappActionStateEmptyElt` |
| `crates/ledger/src/scan_state/transaction_logic/zkapp_command.rs:99` | `ZkappEvents` → `ZkappEventsEmpty` |
| `crates/ledger/src/scan_state/transaction_logic/zkapp_command.rs:120` | `ZkappSeqEvents` → `ZkappActionsEmpty` |
| `crates/ledger/src/scan_state/pending_coinbase.rs:177` | `CoinbaseStack` → `NoInputCoinbaseStack` |
| `crates/ledger/src/account/account.rs` | Removed `#[ignore]` from 3 tests |
| `crates/ledger/src/mask/mask_impl.rs` | Debug `dbg!()` calls (cleanup needed) |
| `crates/ledger/src/sparse_ledger/sparse_ledger_impl.rs` | Debug `dbg!()` calls (cleanup needed) |
| `crates/p2p-messages/src/pseq.rs` | Added `use std::array` (may be unneeded) |

---

## Test Plan

The analysis above contains multiple claims that could be wrong. Each
must be verified independently before trusting the conclusions.

### Test 1: Verify `Fp::from_random_bytes` matches OCaml's `prefix_to_field` — PASSED

**Claim:** `Fp::from_random_bytes(zero_padded_bytes)` produces the same
field element as OCaml's `Field.project(string_bits(s))`.

**How to test:** Write a Rust test that computes the field element for a
known short string (e.g. `"CoinbaseStack"`), and compare against the
OCaml output. Get the OCaml value by running in utop or dune:
```ocaml
Random_oracle.prefix_to_field "CoinbaseStack"
|> Kimchi_pasta_basic.Fp.to_string
```
Then assert the Rust value matches.

**If this fails:** The byte-to-field conversion is different. Possible
causes:
- `from_random_bytes` may reduce mod p differently than `Field.project`
- Bit ordering in `string_bits` may not be LSB-first as assumed
- `from_random_bytes` may hash/scramble input rather than interpret
  directly

Fix: rewrite the field conversion to explicitly do bit projection —
iterate over each byte, extract bits LSB-first, compute
`sum(bit_i * 2^i)` using field arithmetic.

### Test 2: Verify sponge behavior — absorb(1 element) + squeeze — PASSED (partial)

**Claim:** Rust `ArithmeticSponge::absorb(&[x])` followed by
`squeeze()` applies exactly one Poseidon permutation to `[x, 0, 0]` and
returns `state[0]`, matching OCaml's `Make_hash.update` + `digest`.

**Status:** Sponge matches snarky kimchi test vectors (3/3 pass).
Sponge parameters verified to match OCaml (rounds_full=55, alpha=7,
initial_ark=false). Still need OCaml output for end-to-end comparison
of `hash_noinputs("CoinbaseStack")`.

**How to test:** Compute `hash_noinputs` for a known domain string in
Rust and compare against OCaml:
```ocaml
let v =
  Hash_prefix_create.salt "CoinbaseStack"
  |> Random_oracle.digest
in
Kimchi_pasta_basic.Fp.to_string v
```

**If this fails but Test 1 passed:** The sponge operations differ.
Possible causes:
- OCaml pads the input to a full block (rate=2) before permuting, while
  Rust doesn't pad — both should permute `[x, 0, 0]`, but verify by
  comparing intermediate sponge state
- `poseidon_block_cipher` in Rust may differ from OCaml's `block_cipher`
  (round constants, MDS matrix, round count)
- The sponge params (`fp_kimchi::static_params()`) may not match OCaml's
  `Kimchi_pasta_basic.poseidon_params_fp`

Fix: add debug prints in both OCaml and Rust to compare sponge state at
each step: after absorb, after permutation, final output.

### Test 3: Verify the "empty" hash values match OCaml — PENDING

**Claim:** The four "empty" domain strings produce correct hash values.

**How to test:** Get the expected values from OCaml:
```ocaml
(* In mina repo, e.g. via a test or utop *)
let () =
  let open Mina_base in
  let print name v =
    Printf.printf "%s: %s\n" name
      (Kimchi_pasta_basic.Fp.to_string v)
  in
  print "empty_action_state"
    Zkapp_account.Actions.empty_state_element;
  print "empty_events"
    Zkapp_account.Events.empty_hash;
  print "empty_actions"
    Zkapp_account.Actions_impl.empty_hash;
  print "empty_coinbase_stack"
    Pending_coinbase.Coinbase_stack.empty
```

Then write Rust tests asserting each value matches.

**If these fail:** The domain strings may be wrong, or the analysis of
which OCaml string maps to which Rust operation may be incorrect.
Compare the exact string used in each case by adding prints in both
codebases. Don't assume the TEST_FAILURES.md mapping is correct.

### Test 4: Run the previously-failing mina-tree tests — PENDING

Not all workspace crates pass their tests — there are pre-existing
failures unrelated to this PR. Only run the specific crate and tests
that were broken by the domain string issue.

The 51 domain-string failures are all in `mina-tree` (the ledger crate).
Run only that crate:
```bash
cargo nextest run -p mina-tree --lib --no-fail-fast
```

Or for a faster targeted check, run just one representative test from
each failure group:
```bash
# account tests (most common trigger)
cargo nextest run -p mina-tree --lib -E 'test(account::account::tests::test_hash_account)'

# database tests
cargo nextest run -p mina-tree --lib -E 'test(database::database::tests::test_hash_empty)'

# mask tests
cargo nextest run -p mina-tree --lib -E 'test(mask::mask::tests::test_masks)'

# staged_ledger tests
cargo nextest run -p mina-tree --lib -E 'test(staged_ledger::tests_ocaml::max_throughput_normal)'

# scan_state test (different root cause)
cargo nextest run -p mina-tree --lib -E 'test(scan_state::pending_coinbase::tests::test_merkle_tree)'
```

**If 51 domain-string failures are resolved:** The analysis and fix are
correct. Proceed to clean up (remove `dbg!()`, formatting, etc.).

**If some domain-string failures remain:** The fix is incomplete or
wrong. Possible causes:
- There may be additional call sites that pass long domain strings
  through `hash_with_kimchi` (not just `hash_noinputs`). Search:
  ```
  rg "ZkappActionStateEmptyElt|ZkappActionsEmpty\
  |ZkappEventsEmpty|NoInputCoinbaseStack"
  ```
  and verify every usage goes through `hash_noinputs`, never through
  `hash_with_kimchi` or `hash_with_param`.
- The `checked_hash` function (used in `checked_hash_with_param`) may
  also route through `domain_prefix_to_field`. Trace its implementation.
- Expected hash values in tests may have been computed with the OLD
  (incorrect) implementation. If the mechanism is now correct but test
  expectations are wrong, regenerate expected values from OCaml.

**If the 51 are resolved but new failures appear:** The hash values
changed (correctly) but downstream code depended on the old (wrong)
values. Identify which tests fail and whether their expected values need
updating from OCaml.

### Test 5: Address the 2 non-domain-string failures — PENDING

- `test_dummy_sideloaded_verification_key` (account.rs:1967)
- `test_merkle_tree` (pending_coinbase.rs:1265)

These have separate root causes. Investigate independently AFTER the
domain string fix is confirmed working.

### Fallback: If the entire analysis is wrong

If Tests 1-3 all fail, the fundamental assumption about OCaml/Rust
equivalence is wrong. In that case:
1. Add extensive debug logging to both OCaml and Rust sponge operations
2. Hash a single known field element (e.g. `Fp::from(42u64)`) through
   both implementations and compare intermediate states step by step
3. Check whether the Poseidon round constants and MDS matrices are
   identical between `mina_poseidon::pasta::fp_kimchi::static_params()`
   and OCaml's `Kimchi_pasta_basic.poseidon_params_fp`
4. Consider that `proof-systems` tag `0.3.0` may be out of sync with
   the OCaml proof-systems version used by mina

---

## Key Reference Files

| File | What to look at |
|------|----------------|
| `mina/src/lib/hash_prefixes/hash_prefixes.ml` | 20-byte limit, star-padding, `create` function |
| `mina/src/lib/random_oracle/random_oracle.ml:95-100` | `prefix_to_field` — 31-char limit, bit projection |
| `mina/src/lib/random_oracle/random_oracle.ml:100` | `salt` — absorbs prefix_to_field into sponge |
| `mina/src/lib/hash_prefix_states/hash_prefix_states.ml:4` | `salt(s)` — takes a `Hash_prefixes.t` (already star-padded) |
| `mina/src/lib/mina_base/zkapp_account.ml:97-126` | OCaml "empty" domain strings |
| `mina/src/lib/mina_base/pending_coinbase.ml:186` | OCaml empty coinbase stack |
| `snarky/sponge/sponge.ml:208-256` | `Make_hash` — `update`, `digest`, `to_blocks` |
| `snarky/sponge/fold_lib/fold.ml:105-110` | `string_bits` — LSB-first bit extraction |
| `proof-systems/hasher/src/lib.rs:169-183` | Rust `domain_prefix_to_field` — 20-char assert |
| `proof-systems/hasher/src/poseidon.rs:86-104` | Rust hasher `init` — absorb + squeeze |
| `proof-systems/poseidon/src/poseidon.rs:79-146` | Rust `ArithmeticSponge` — absorb/squeeze impl |

---

## Verification Results

### Sponge Parameters Match (VERIFIED)

Traced the full OCaml module chain:
1. `Random_oracle` → `Sponge.Make_hash(Random_oracle_permutation)`
2. `Random_oracle_permutation` → `Sponge.Poseidon(Pickles.Tick_field_sponge.Inputs)`
3. `Tick_field_sponge` = `Make_sponge.Make(Backend.Tick.Field)` with `params = Kimchi_pasta_basic.poseidon_params_fp`
4. `Make_sponge.Rounds`: `rounds_full = 55`, `initial_ark = false`, `rounds_partial = 0`
5. `Make_sponge.Inputs.alpha = 7`

Rust `PlonkSpongeConstantsKimchi` (proof-systems/poseidon/src/constants.rs):
- `PERM_ROUNDS_FULL = 55` ✓
- `PERM_SBOX = 7` (alpha) ✓
- `PERM_INITIAL_ARK = false` ✓
- `PERM_ROUNDS_PARTIAL = 0` ✓
- `SPONGE_RATE = 2`, `SPONGE_WIDTH = 3`, `SPONGE_CAPACITY = 1` ✓

### Sponge Test Vectors Pass (VERIFIED)

Validated Rust `ArithmeticSponge` against snarky kimchi test vectors
(`snarky/sponge/test_vectors/kimchi.json`):
- Empty input: `a8eb9ee0f300...` ✓
- 1 input: `fb5992f65c07...` ✓
- 2 inputs: `fe2436f20276...` ✓

Test added: `crates/ledger/src/hash.rs::tests::test_sponge_kimchi_vectors`

### Byte-to-field Conversion (VERIFIED)

`Fp::from_random_bytes("CoinbaseStack" ++ zeros)` produces field element
`436f696e62617365537461636b000...000` — matches the raw LE byte
interpretation, same as OCaml's `Field.project(string_bits(s))`.

Test added: `crates/ledger/src/hash.rs::tests::test_hash_noinputs_consistency`

### hash_noinputs Output

`hash_noinputs("CoinbaseStack")` =
`35b9d51e5d7c741456f86720731241a8280273cfc6c21668fd7bc6c587d0cc1d`

**This value needs to be compared against OCaml output to complete
end-to-end verification.** Getting the OCaml value requires running:
```ocaml
Hash_prefix_create.salt "CoinbaseStack" |> Random_oracle.digest
|> Kimchi_pasta_basic.Fp.to_hex
```

### Remaining Work

1. Get OCaml hash outputs for the 4 "empty" domain strings to confirm
   end-to-end match
2. Run the full mina-tree test suite to see how many of the 51 failures
   are resolved
3. Clean up debug artifacts (dbg!() calls)
4. Address 2 non-domain-string test failures separately

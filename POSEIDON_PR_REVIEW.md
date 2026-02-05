# PR Review Workflow

- Fix comments one at a time, track progress for comments addressed as well as files where all comments have been addressed.
- When addressing a comment, first ask a clarifying question. Always ask a clarifying question. Then come up with a plan to address the comment. Wait for approval of the plan to execute.
- After implementing the fix, confirm with the user that the comment has been addressed. Once confirmed, update the status on the pr-review markdown file.
- ALWAYS use `rg`. NEVER use `grep`.
- ALWAYS use `fd`. NEVER use `find`.
- For any dependencies, ask the user to build the dependency locally, then use the rustdocs in the `target` directory for any usage of dependencies and to see what APIs are available.

---

# Poseidon PR Review Progress (#2134)

## File Progress

- [ ] POSEIDON_EXECUTION_PLAN.md
- [ ] POSEIDON_MIGRATION_PLAN.md
- [ ] Cargo.toml
- [ ] crates/ledger/Cargo.toml
- [ ] crates/ledger/src/common.rs
- [ ] crates/ledger/src/hash.rs
- [ ] crates/ledger/src/proofs/opt_sponge.rs
- [x] crates/ledger/src/proofs/poseidon_params.rs
- [ ] crates/ledger/src/proofs/step.rs
- [ ] crates/ledger/src/proofs/transaction.rs
- [ ] crates/ledger/src/scan_state/transaction_logic/transaction_union_payload.rs
- [ ] crates/ledger/src/verifier/common.rs
- [ ] crates/node/src/transition_frontier/genesis/transition_frontier_genesis_config.rs
- [ ] crates/p2p-messages/src/hash_input.rs
- [ ] crates/p2p-messages/src/v2/hashing.rs
- [ ] crates/snark/src/lib.rs
- [ ] crates/snark/src/merkle_path/mod.rs
- [ ] crates/vrf/src/lib.rs
- [ ] crates/vrf/src/message.rs
- [ ] crates/vrf/src/output.rs
- [ ] poseidon/src/lib.rs

## Comment Progress

### crates/ledger/src/proofs/poseidon_params.rs
- [x] `2755060231`: "🤔 this is already in the mina-poseidon crate..." (line 1)

### crates/ledger/src/proofs/transaction.rs
- [ ] `2755065161`: "This could be a const" (line 1774)
    - *Note: Refactoring to associated constants. PERM_ROUNDS_FULL for Legacy is currently 63, but LEGACY_ROUNDS is 100. Leaving at 63 for now but may need to sync later.*
- [ ] `2766024062`: "These probably shouldn't be pub" (line 1589)
- [ ] `2766025289`: "Comment on sizes here" (line 1602)
- [ ] `2766032285`: "This is kind of crazy too. Can and should use bytemuck to do something more efficient" (line 1589)
- [ ] `2766034963`: "super inefficient" (line 1627)
- [ ] `2766042063`: "This trait definitely needs docs" (line 1665)
- [ ] `2766045464`: "FIX" (line 1780)
- [ ] `2766050755`: "I don't think this is even sound. The size of the legacy params is way bigger and the number of rounds is completely different" (line 1791)
- [ ] `2766061968`: "These are now the same function. The reason `new_with_state_params` was there before was for using with legacy params. We could still put a pointer in here though... TBD" (line 1825)
- [ ] `2766066183`: "Have to look how the witness is different here vs `mina_poseidon`" (line 1993)
- [ ] `2766069650`: "Again, need confirmation on what's different here" (line 2084)
- [ ] `2766076306`: "The sponge probably shouldn't have an array and be owned instead..." (line 2704)
- [ ] `2766083850`: "Shouldn't have "floating" static strs, should make a constant" (line 3034)
- [ ] `2766086087`: "How is this "checked"?" (line 3868)
- [ ] `2766090467`: "Should probably get this attribute in `develop` in a separate PR and enable the `mina-tree` tests in CI" (line 4606)
- [ ] `2766092627`: "This should be an enum instead of strings" (line 4854)
- [ ] `2766158667`: "ah, I just forgot to download the file that make this test work" (line 4606)
- [ ] `2766159711`: "This should be an enumj instead of a str" (line 4858)

### crates/snark/src/merkle_path/mod.rs
- [ ] `2755082476`: "sus that this was deleted (the whole test that is)" (line 75)
- [ ] `2766311490`: "Yeah, this test definitely needs to be re-enabled. The question is, was it correct to begin with." (line 82)

### crates/snark/src/lib.rs
- [ ] `2755085246`: "🤔" (line 65)

### crates/vrf/src/lib.rs
- [ ] `2755087653`: "Now remember to delete the comment" (line 286)
- [ ] `2766313010`: "deleteme" (line 256)
- [ ] `2766313950`: "Why is this being cloned" (line 238)

### poseidon/src/lib.rs
- [ ] `2755093879`: "There shouldn't be any changes in here, this whole file/crate should be deleted" (line 1)

### POSEIDON_EXECUTION_PLAN.md
- [ ] `2755095620`: "delete me" (line 1)

### POSEIDON_MIGRATION_PLAN.md
- [ ] `2755096472`: "delete me" (line 1)

### crates/ledger/src/proofs/opt_sponge.rs
- [ ] `2764880949`: "Double check this" (line 162)
- [ ] `2764883319`: "And here... but does it make sense that this even has to be re-implemented?" (line 263)
- [ ] `2764889199`: "This is odd. This is just a constant based on F. It should be a different generic argument, I would think, not a trait bound on F." (line 279)
- [ ] `2764891882`: "This should already exist in `mina_poseidon`" (line 301)
- [ ] `2764894348`: "`sbox` should already exist in mina-poseidon. How is this different?" (line 282)

### crates/ledger/src/proofs/step.rs
- [ ] `2764897875`: "no unwraps" (line 2033)
- [ ] `2764899182`: "...this is odd that we have to do this" (line 2715)

### crates/ledger/src/scan_state/transaction_logic/transaction_union_payload.rs
- [ ] `2766163214`: "This is duplicate code" (line 346)
- [ ] `2766163955`: "this is all duplicate code" (line 410)

### crates/ledger/src/verifier/common.rs
- [ ] `2766180139`: "Why not use `mina_core` here? They should honestly be the same type" (line 36)
- [ ] `2766182192`: "Same comment, I think we can cahnge this so that we only use `mina_core`" (line 213)

### crates/ledger/src/common.rs
- [ ] `2766188599`: "Is this used anywhere? It's a private field and the `Hashable` trait takes an argument for its `domain_string` static function." (line 21)
- [ ] `2766194888`: "This is a weird type. The `check` function should return a `Result` with an enum of valid results and a separate enum that covers all the `Err` cases." (line 54)
- [ ] `2766199502`: "Need to stop putting `use` statements inside functions. We should be defaulting to putting them at the top of the module (whether the module is the file or whether it's defined in the file)" (line 58)
- [ ] `2766203418`: "Was this file just moved?" (line 1)
- [ ] `2766211020`: "suggestion for append_field" (line 213)
- [ ] `2766213230`: "Again, I don't think this field is actually being used anywhere" (line 217)
- [ ] `2766214037`: "suggestion for Fq::from" (line 221)
- [ ] `2766214741`: "suggestion for CurvePoint::generator" (line 223)

### crates/ledger/src/hash.rs
- [ ] `2766219803`: "`ROInput` shouldn't be pub. If we need to, we should use a constructor and an `into_inner` method, but I don't think we want to be able to mutate the wrapped `inner`." (line 22)
- [ ] `2766223702`: "None of these need to be cloned." (line 50)
- [ ] `2766228095`: "Where are all of these being used? This would probably work better as an enum, especially since we are converting an integer into the string below." (line 138)
- [ ] `2766230557`: "I could be convinced that this is the best way though" (line 138)
- [ ] `2766238074`: "This should be generic: a single function with a field type. we can create a private trait here to get the static params properly" (line 221)
- [ ] `2766243817`: "We should be abel to change the name of this to something like `hash_with_domain`. Here, it would also be nice if the domain was a concrete type (not sure if this is possible though)" (line 249)
- [ ] `2766249237`: "If it's impossible to make this test fail, we should just get rid of the test." (line 330)

### crates/ledger/Cargo.toml
- [ ] `2766253224`: "bad" (line 114)

### crates/node/src/transition_frontier/genesis/transition_frontier_genesis_config.rs
- [ ] `2766258635`: "What's the type of `PROTOCOL_CONSTANTS`? Why does this need to be cloned?" (line 268)

### crates/p2p-messages/src/v2/hashing.rs
- [ ] `2766266665`: "We shouldn't have unwraps" (line 882)
- [ ] `2766268404`: "..." (line 956)

### crates/p2p-messages/src/hash_input.rs
- [ ] `2766271919`: "Can we get a github permalink to the reference implementation in ocaml?" (line 55)
- [ ] `2766274207`: "Can we get a permalink to the ocaml code" (line 67)
- [ ] `2766278972`: "All these tests are testing inputs which are intermediate values. We shouldn't be testing this, we should be testing the final hash to see if it's correct, that's it. We should be using the same test cases that exist in the ocaml codebase." (line 151)

### crates/vrf/src/message.rs
- [ ] `2766316046`: "We should probably panic here for now instead of fail silently, right?" (line 48)
- [ ] `2766319897`: "Could probably use a permalink to the mina ocaml code here as well" (line 66)

### crates/vrf/src/message.rs
- [ ] `2766321902`: "permalink to hte ocaml code, please" (line 52)
- [ ] `2766324710`: "Why do we have to unwrap here? At worst, we should use an `expect` with a big `// SAFETY` comment." (line 55)
- [ ] `2766326690`: "Why were these tests deleted" (line 164)

### Cargo.toml
- [ ] `2766328448`: "Need to remove the two comment lines above this as well" (line 24)

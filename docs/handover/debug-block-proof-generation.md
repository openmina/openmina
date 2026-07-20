# Instructions on using the failed block proof dumps for debugging block proofs

1. First save our private rsa key:

```
$ cp private_key $HOME/.openmina/debug/rsa.priv
```

2. To decrypt producer private key:

```
$ cp failed_block_proof_input_$HASH.binprot /tmp/block_proof.binprot
$ cd openmina/ledger
$ cargo test --release add_private_key_to_block_proof_input -- --nocapture
# This create the file /tmp/block_proof_with_key.binprot
```

3. Run proof generation in Rust:  
   Apply those changes to the test `test_block_proof`:

```diff
modified   ledger/src/proofs/transaction.rs
@@ -4679,10 +4679,11 @@ pub(super) mod tests {
     #[test]
     fn test_block_proof() {
         let Ok(data) = std::fs::read(
-            Path::new(env!("CARGO_MANIFEST_DIR"))
-                .join(devnet_circuit_directory())
-                .join("tests")
-                .join("block_input-2483246-0.bin"),
+            "/tmp/block_proof_with_key.binprot"
         ) else {
             eprintln!("request not found");
             panic_in_ci();
@@ -4690,7 +4691,8 @@ pub(super) mod tests {
         };

         let blockchain_input: v2::ProverExtendBlockchainInputStableV2 =
-            read_binprot(&mut data.as_slice());
+            v2::ProverExtendBlockchainInputStableV2::binprot_read(&mut data.as_slice()).unwrap();
+            // read_binprot(&mut data.as_slice());

         let BlockProver {
             block_step_prover,
```

Then you can run:

```
$ cd openmina/ledger
$ cargo test --release test_block_proof -- --nocapture
```

4. Run proof generation in OCaml:  
   Use this branch: https://github.com/openmina/mina/tree/proof-devnet

```
$ cd mina
$ export CC=gcc CXX=g++ RUST_BACKTRACE=1 DUNE_PROFILE=devnet
$ make build && _build/default/src/app/cli/src/mina.exe internal run-prover-binprot < /tmp/block_proof_with_key.binprot 2>&1 | tee /tmp/LOG.txt
```

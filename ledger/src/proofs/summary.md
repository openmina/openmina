# Proofs Module Summary

The proofs module handles zero-knowledge proof generation and verification for
the Mina protocol using the Kimchi proof system. The implementation has proven
to work reliably on devnet but contains many TODOs and probably cleanup.

## Quick Reference

**Core Proof Types**

- `transaction.rs` - Transaction verification with witness generation
- `block.rs` - Blockchain state transitions with consensus validation
- `merge.rs` - Combining multiple transaction proofs
- `zkapp.rs` - Smart contract execution proofs with authorization types

**Infrastructure**

- `witness.rs` - Witness data management (primary/auxiliary)
- `caching.rs` - Verifier index and SRS caching
- `constants.rs` - Circuit sizes and domain configurations
- `step.rs`/`wrap.rs` - Step/wrap proof pattern for recursion

## Implementation

**Kimchi Integration**

- Type aliases in `mod.rs` directly use Kimchi types for verifier/prover indices
  and proofs
- Circuit constraints and field operations built on Kimchi foundations
- Maintains compatibility with Mina protocol proof formats
- Uses a fork of proof-systems that is based on an older version than currently
  used by OCaml implementation
- Uses a fork of arkworks to considerably speed up field operations on WASM
- See [#1106](https://github.com/o1-labs/openmina/issues/1106) for details on
  these forks

**Performance Features**

- Caching system stores verifier indices and SRS data in `$HOME/.cache/openmina`
- Circuit blobs for external circuit data fetching
- Precomputed verification indices for devnet/mainnet in `data/` directory

**Pickles Recursive Proof System**

- The entire proofs module implements Pickles (recursive proof composition
  system)
- `step.rs`/`wrap.rs` provide the fundamental step/wrap recursion pattern
- `public_input/prepared_statement.rs` handles different recursion levels (N0,
  N1, N2)

**Witness vs Circuit Split**

- Witness generation handled in `witness.rs` with comparison functionality for
  OCaml compatibility testing
- Circuit logic implemented but lacks constraint declarations and
  compilation/evaluation functionality
- Uses precomputed verification indices from `data/` directory

For details on missing constraint functionality and circuit management, see
[`docs/handover/circuits.md`](../../../docs/handover/circuits.md).

## Known Issues

**Incomplete Functionality**

- Joint combiner functionality has TODO items in `step.rs`
- Feature flag handling incomplete in some verification paths
- ZkApp call stack hash computation needs completion
- Some field type conversions marked as temporary hacks

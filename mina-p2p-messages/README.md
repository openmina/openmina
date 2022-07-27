# Mina Gossip Rust Types

This library contains Rust implementation of types used in Mina blockchain
gossip messages.

This types are generated from `bin_prot` shapes.

## Types Generation

TODO

``` sh
cargo run --bin mina-types -- ../mina/shapes-raw.txt gen \
   --type Mina_block__Block.Stable.V2.t \
   --type Network_pool__Transaction_pool.Diff_versioned.Stable.V2.t \
   --type Network_pool__Snark_pool.Diff_versioned.Stable.V2.t \
   --config default.toml \
   --out src/lib.rs
```

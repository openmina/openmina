# Mina Gossip Rust Types

This library contains Rust implementation of types used in Mina blockchain
gossip messages.

This types are generated from `bin_prot` shapes.

## Decoding `bin_prot` Stream

To decode a binary gossip message, one of the following types should be used:
- `mina_p2p_messages::p2p::MinaBlockExternalTransitionRawVersionedStable` for an
  External Transition message
- `mina_p2p_messages::p2p::NetworkPoolSnarkPoolDiffVersionedStable` for a Snark
  Pool Diff message
- `mina_p2p_messages::p2p::NetworkPoolTransactionPoolDiffVersionedStable` for a
  Transaction Pool Diff message

There is also the `mina_p2p_messages::GossipNetMessage` enum type that
corresponds to the OCaml `Gossip_net.Message.V1.T.msg` type that sums up all
three kinds of messages to be used over the wire.

Each message implement `binprot::BinProtRead` trait, so e.g. for reading an
external transition, use the following:

``` rust
    let external_transition =
        MinaBlockExternalTransitionRawVersionedStable::binprot_read(&mut ptr)?;
```

All types implement `serde` serialization, so they can be easily turned into
JSON:

``` rust
   let external_transition_json = serde_json::to_string(&external_transition)?;
```

## Types Generation

TODO

``` sh
cargo run --bin mina-types -- ../mina/shapes-raw.txt gen \
   --type Mina_block__External_transition.Raw_versioned__.Stable.V1.t \
   --type Network_pool__Transaction_pool.Diff_versioned.Stable.V1.t \
   --type Network_pool__Snark_pool.Diff_versioned.Stable.V1.t \
   --config default.toml \
   --out src/lib.rs
```

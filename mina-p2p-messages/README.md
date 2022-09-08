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

The `bin-proto-rs` crate is used to automatically generate Mina wire types
basing on their bin_prot shapes, stored in [shapes](shapes) folder. Currently
only subset of types can be generated. Types currently known as essential are
listed in the files [types-v1.txt](types-v1.txt) and
[types-v2.txt](types-v2.txt).

To generate Mina V1 types, use the following command:

``` sh
mina-types shapes/shapes-v1-mainnet-raw.txt gen \
   $(cat types-v1.txt)
   --config default.toml \
   --out src/v1.rs
```

To generate Mina V2 types, use the following command:

``` sh
mina-types shapes/shapes-v2-mainnet-raw.txt gen \
   $(cat types-v2.txt)
   --config default.toml \
   --out src/v2.rs
```

#![allow(unexpected_cfgs)]
#![cfg(benchmarks)]
#![feature(test)]

use binprot::{BinProtRead, BinProtWrite};
use test::Bencher;

extern crate test;

mod utils;
use utils::*;

#[bench]
fn decode_v2(b: &mut Bencher) {
    let binary = read("v2/gossip/new_state.bin").unwrap();
    b.iter(|| {
        let _ = mina_p2p_messages::gossip::GossipNetMessageV2::binprot_read(&mut binary.as_slice())
            .unwrap();
    })
}

#[bench]
fn encode_v2(b: &mut Bencher) {
    let binary = read("v2/gossip/new_state.bin").unwrap();
    let t = mina_p2p_messages::gossip::GossipNetMessageV2::binprot_read(&mut binary.as_slice())
        .unwrap();
    let mut buf = Vec::with_capacity(binary.len());
    b.iter(|| {
        let _ = t.binprot_write(&mut buf).unwrap();
        buf.clear();
    })
}

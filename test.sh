#!/bin/bash

export MINA_DISCOVERY_FILTER_ADDR=false
export KEEP_CONNECTION_WITH_UNKNOWN_STREAM=true
export REPLAYER_MULTIADDR=/dns4/mina-rust-ci-1-libp2p.gcp.o1test.net/tcp/8302/p2p/12D3KooWNazk9D7RnbHFaPEfrL7BReAKr3rDRf7PivS2Lwx3ShAA
export BPF_ALIAS=/coda/0.0.1/29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6-0.0.0.0


cargo test --release             --package=mina-node-testing             --package=cli -- --exact bootstrap_from_replayer --nocapture
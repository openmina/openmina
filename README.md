
# The Open Mina Node

[![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml)

The Open Mina Node is a Mina node written completely in Rust and capable of verifying blocks of transactions, producing blocks and generating SNARKs. 

In the design of the Open Mina node, we are utilizing much of the same logic as in the Mina Web Node. The key difference is that unlike the Web Node, which is an in-browser node with limited resources, the Open Mina node is able to perform resource-intensive tasks such as SNARK proof generation.


## Overview of the Node’s current functionalities

Currently, with the Open Mina node, you can:



* Verify blocks and transactions
* Produce SNARKs
* Broadcast messages: block information, transaction pool, SNARK pool


We are working on the following:


* Produce SNARKs in Rust (currently we use OCaml subprocess for that)


In the future, we plan to implement:


* Direct transfer of MINA funds 
* Block production
* SNARK proof generation for transactions
* SnarkyJS support for Rust node, thanks to which you will be able to directly inject simple transactions, such as transferring Mina funds from one account to another.
* The ability to record/replay all blocks 
* A user interface (UI) for the node. Through the UI, users will be able to control the node and get information about its status.


## How to launch:

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

**Pre-requisites:**

Ubuntu or Debian-based Linux distribution

**Steps (for Debian-based Linux distros):**

Open up the command line and enter the following:


```
apt install curl git

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2023-10-07

source "$HOME/.cargo/env"

git clone https://github.com/openmina/openmina.git

cd openmina/

git submodule update --init

cargo run --release -p cli node


# The Open Mina documentation overview

- [Why we are developing Open Mina](https://github.com/JanSlobodnik/pre-publishing/blob/main/why-openmina.md)
- What is Open Mina?
  - [Openmina Node](https://github.com/openmina/openmina/blob/main/README.md)
  - [The Mina Web Node](https://github.com/openmina/webnode/blob/main/README.md)
- Core components
  - [P2P communication](https://github.com/openmina/openmina/blob/documentation/docs/p2p_service.md)
    - [GossipSub](https://github.com/openmina/mina-wiki/blob/3ea9041e52fb2e606918f6c60bd3a32b8652f016/p2p/mina-gossip.md)
  - [Block production](https://github.com/JanSlobodnik/pre-publishing/blob/main/block-production.md)
  - [Scan state](https://github.com/JanSlobodnik/pre-publishing/blob/main/scan-state.md)
  - [SNARKs](https://github.com/JanSlobodnik/pre-publishing/blob/main/SNARKs.md)
  - Developer tools
    - [Debugger](https://github.com/openmina/mina-network-debugger/blob/main/README.md)
  - [Front End](https://github.com/openmina/mina-frontend/blob/main/README.md)
    - [Dashboard](https://github.com/openmina/mina-frontend/blob/main/docs/MetricsTracing.md#Dashboard)
- [Testing](https://github.com/openmina/openmina/blob/develop/testing.md)
- How to run
  - [Openmina node](https://github.com/openmina/openmina/blob/main/README.md)
  - [Debugger](https://github.com/openmina/mina-network-debugger?tab=readme-ov-file#Preparing-for-build)
  - [Web Node](https://github.com/openmina/webnode/blob/main/README.md#try-out-the-mina-web-node)
- External links
  - [Substack](https://openmina.substack.com/)
  - [Medium](https://medium.com/openmina)
  - [Social media](https://twitter.com/viable_systems)

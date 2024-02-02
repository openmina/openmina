
# The Open Mina Node

[![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml) [![Changelog][changelog-badge]][changelog] [![release-badge]][release-link] [![Apache licensed]][Apache link]

The Open Mina Node is a Mina node written completely in Rust and capable of verifying blocks of transactions, producing blocks and generating SNARKs.

In the design of the Open Mina node, we are utilizing much of the same logic as in the Mina Web Node. The key difference is that unlike the Web Node, which is an in-browser node with limited resources, the Open Mina node is able to perform resource-intensive tasks such as SNARK proof generation.


## Overview of the Node’s current functionalities

Currently, with the Open Mina node, you can:



* Connect to the network and sync up to the best tip block
* Validate and apply new blocks and transactions to update consensus and ledger state.
* Produce SNARKs to complete SNARK work.
* Broadcast messages: blocks, SNARK pool


We are working on the following:


* Produce SNARKs in Rust (currently we use OCaml subprocess for that)


In the future, we plan to implement:

* Direct transfer of MINA funds
* Block production
* SNARK proof generation for transactions
* SnarkyJS support for Rust node, thanks to which you will be able to directly inject simple transactions, such as transferring Mina funds from one account to another.
* The ability to record/replay all blocks


## How to launch (with docker compose):

Run:

```
docker compose up
```

Then visit http://localhost:8070 in your browser.

## How to launch (without docker compose):

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

**Pre-requisites:**

Ubuntu or Debian-based Linux distribution.

**Steps (for Debian-based Linux distros):**

Open up the command line and enter the following:


``` sh
apt install curl git

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2023-10-07

source "$HOME/.cargo/env"

git clone https://github.com/openmina/openmina.git

cd openmina/

cargo run --release -p cli node
```

## How to launch the UI:

### Prerequisites

The following tools are required to launch the UI:

- Node.js
- npm
- Angular CLI

Here are the steps to install these tools:

#### 1. Node.js

Download and install [Node.js](https://nodejs.org/) for your OS, which includes Node Package Manager (npm).

- After installing Node.js, verify your installation:

  ```bash
  node -v
  ```
  This command should print the version number of your Node.js installation.

#### 2. npm
- Verify that you are running a version of npm that is at least 6.x.x or higher:

  ```bash
  npm -v
  ```
  This command should print the version number of your npm installation.

#### 3. Angular CLI
- Install the Angular CLI globally:

  ```bash
  npm install -g @angular/cli
  ```
  This command installs the Angular CLI globally on your system.
- Verify your Angular CLI installation:

  ```bash
  ng version
  ```
  This command should print the version number of your Angular CLI installation.

### Steps (for any OS)

Open the command line, navigate to the openmina directory and then run

``` sh
cd frontend
npm install
npm start
```

Open your browser and navigate to [http://localhost:4200](http://localhost:4200).

## Repository Structure

- [core/](core) - Provides basic types needed to be shared across different
  components of the node.
- [ledger/](ledger) - Mina ledger implementation in Rust.
- [snark/](snark) - Snark/Proof verification.
- [p2p/](p2p) - P2p implementation for OpenMina node.
- [node/](node) - Combines all the business logic of the node.
  - [native/](node/native) - OS specific pieces of the node, which is
    used to run the node natively (Linux/Mac/Windows).
  - [testing/](node/testing) - Testing framework for OpenMina node.
- [cli/](cli) - OpenMina cli.
- [frontend/](frontend) - OpenMina frontend.


[Details regarding architecture](ARCHITECTURE.md)


## The Open Mina Documentation

- [Why we are developing Open Mina](docs/why-openmina.md)
- What is Open Mina?
  - [Openmina Node](#the-open-mina-node)
  - [The Mina Web Node](https://github.com/openmina/webnode/blob/main/README.md)
- Core components
  - [P2P communication](https://github.com/openmina/openmina/blob/documentation/docs/p2p_service.md)
    - [GossipSub](https://github.com/openmina/mina-wiki/blob/3ea9041e52fb2e606918f6c60bd3a32b8652f016/p2p/mina-gossip.md)
  - [Scan state](docs/scan-state.md)
  - [SNARKs](docs/snark-work.md)
- Developer tools
  - [Debugger](https://github.com/openmina/mina-network-debugger/blob/main/README.md)
  - [Front End](https://github.com/openmina/mina-frontend/blob/main/README.md)
    - [Dashboard](https://github.com/openmina/mina-frontend/blob/main/docs/MetricsTracing.md#Dashboard)
- [Testing](docs/testing/testing.md)
- How to run
  - [Launch Openmina node](#how-to-launch-without-docker-compose)
  - [Launch Node with UI](#how-to-launch-with-docker-compose)
  - [Debugger](https://github.com/openmina/mina-network-debugger?tab=readme-ov-file#Preparing-for-build)
  - [Web Node](https://github.com/openmina/webnode/blob/main/README.md#try-out-the-mina-web-node)
- External links
  - [Medium](https://medium.com/openmina)
  - [Twitter](https://twitter.com/viable_systems)


[changelog]: ./CHANGELOG.md
[changelog-badge]: https://img.shields.io/badge/changelog-Changelog-%23E05735

[release-badge]: https://img.shields.io/github/v/release/openmina/openmina
[release-link]: https://github.com/openmina/openmina/releases/latest

[Apache licensed]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[Apache link]: https://github.com/openmina/openmina/blob/master/LICENSE

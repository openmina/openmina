
# The OpenMina Node

[![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml) [![Changelog][changelog-badge]][changelog] [![release-badge]][release-link] [![Apache licensed]][Apache link]

The OpenMina Node is a Mina node written entirely in Rust, capable of verifying blocks of transactions, producing blocks and generating SNARKs.

Unlike the resource-limited Mina Web Node, OpenMina handles resource-intensive tasks like SNARK proof generation.

| The OpenMina node allows you to                                                             | In Development                                               | Future Plans                                                                                                                |
|---------------------------------------------------------------------------------------------|--------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|
| &#9745; **Connect** to the network and sync up to the best tip block                              | &#9744; Produce SNARKs in Rust (currently we use OCaml subprocess for that) | &#9744; Direct transfer of MINA funds                                                                                          |
| &#9745; **Validate** and apply new blocks and transactions to update consensus and ledger state   |                                                              | &#9744; Block production                                                                                                       |
| &#9745; **Produce SNARKs** to complete SNARK work                                                 |                                                              | &#9744; The ability to record/replay all blocks                                                                                      |
| &#9745; **Broadcast** messages: blocks, SNARK pool                                                |                                                              | &#9744; SnarkyJS support for Rust node, enabling direct injection of simple transactions, such as transferring Mina funds      |
| &#9745; **SNARK proof generation** for transactions                                                                                             |                                                              |                                                                          |


## Table of Contents
- Try Yourself
  - [Block Producer Demo](#producer-demo)
  - [Quick-start a Node with Docker Compose](#)
  - [Building a Node from Source](#)
- Learn about the Repository
  - [Repository Structure](#repository-structure)
  - [Documentation](#documentation)
  - [License](#license)

## Block Producer Demo
This demo runs in a private network with block proofs disabled, eliminating the need to wait for staking ledger inclusion.

[Follow the detailed guide](docs/producer-demo.md).

## Quick-start a Node with Docker Compose

#### 1. Run:
```
docker compose up
```
#### 2. Visit http://localhost:8070 in your browser.
- By default, `docker compose up` will use the latest node and frontend images available (tagged with `latest`), but specific versions can be selected by using the `OPENMINA_TAG` and `OPENMINA_FRONTEND_TAG` variables.
- The node will be running on the Berkeley network.

## Building a Node from Source

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

### 1. Install node pre-requisites:

Ubuntu or Debian-based Linux distribution with the following packages installed:
- `curl`
- `git`
- `libssl-dev`
- `pkg-config`
- `protobuf-compiler`
- `build-essential`

####  MacOS

If Homebrew is not installed:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```
``` sh
brew install curl git openssl pkg-config protobuf gcc make
```

#### Linux (Debian-based)
``` sh
# Either using "sudo" or as the "root" user
sudo apt install curl git libssl-dev pkg-config protobuf-compiler build-essential
```

### 2. Build and Run the Node:
**Steps (for Debian-based Linux distros and macOS):**

Open up the command line and enter the following:
```sh
# Install rustup and set the default Rust toolchain to 1.77 (newer versions work too)
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.77
# Setup the current shell with rustup
source "$HOME/.cargo/env"
# Clone the openmina repository
git clone https://github.com/openmina/openmina.git
cd openmina/
# Build and run the node
cargo run --release -p cli node
```

### 3. Install Dashboard pre-requisites:

#### 3A. Node.js v20.11.1

##### MacOS
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node@20.11.1
```
##### Linux (Debian-based)
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
source ~/.bashrc
nvm install 20.11.1
```
##### Windows
Download [Node.js v20.11.1](https://nodejs.org/) from the official website, open the installer and follow the prompts to complete the installation.

#### 3B. Angular CLI v16.2.0
```bash
npm install -g @angular/cli@16.2.0
```

### 4. Build and Run Dashboard Application

#### 4A. Open a terminal and navigate to this project's root directory:
```bash
cd PROJECT_LOCATION/openmina/frontend
```
#### 4B. Install the dependencies:
```bash
npm install
```

#### 4C. Run the application:

```bash
npm start
```
<kbd> ![Image](https://github.com/openmina/mina-frontend/assets/1679939/40428a22-0691-473a-a15c-c2e610a2217b)

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

---

## Block Producer Demo
This demo runs in a private network with block proofs disabled, eliminating the need to wait for staking ledger inclusion.

[Follow the detailed guide](docs/producer-demo.md).

## Quick-start a Node with Docker Compose

#### 1. Run:
```
docker compose up
```
#### 2. Visit http://localhost:8070 in your browser.
- By default, `docker compose up` will use the latest node and frontend images available (tagged with `latest`), but specific versions can be selected by using the `OPENMINA_TAG` and `OPENMINA_FRONTEND_TAG` variables.
- The node will be running on the Berkeley network.

## Building a Node from Source

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

### 1. Install node pre-requisites:

Ubuntu or Debian-based Linux distribution with the following packages installed:
- `curl`
- `git`
- `libssl-dev`
- `pkg-config`
- `protobuf-compiler`
- `build-essential`

####  MacOS

If Homebrew is not installed:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```
``` sh
brew install curl git openssl pkg-config protobuf gcc make
```

#### Linux (Debian-based)
``` sh
# Either using "sudo" or as the "root" user
sudo apt install curl git libssl-dev pkg-config protobuf-compiler build-essential
```

### 2. Build and Run the Node:
**Steps (for Debian-based Linux distros and macOS):**

Open up the command line and enter the following:
```sh
# Install rustup and set the default Rust toolchain to 1.77 (newer versions work too)
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.77
# Setup the current shell with rustup
source "$HOME/.cargo/env"
# Clone the openmina repository
git clone https://github.com/openmina/openmina.git
cd openmina/
# Build and run the node
cargo run --release -p cli node
```

### 3. Install Dashboard pre-requisites:

#### 3A. Node.js v20.11.1

##### MacOS
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node@20.11.1
```
##### Linux (Debian-based)
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
source ~/.bashrc
nvm install 20.11.1
```
##### Windows
Download [Node.js v20.11.1](https://nodejs.org/) from the official website, open the installer and follow the prompts to complete the installation.

#### 3B. Angular CLI v16.2.0
```bash
npm install -g @angular/cli@16.2.0
```

### 4. Build and Run Dashboard Application

#### 4A. Open a terminal and navigate to this project's root directory:
```bash
cd PROJECT_LOCATION/openmina/frontend
```
#### 4B. Install the dependencies:
```bash
npm install
```

#### 4C. Run the application:

```bash
npm start
```
<kbd> ![Image](https://github.com/openmina/mina-frontend/assets/1679939/40428a22-0691-473a-a15c-c2e610a2217b)

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

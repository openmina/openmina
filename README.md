
# The Open Mina Node
## With the Rust-based Open Mina node, you can produce, validate and apply blocks

[![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml) [![Changelog][changelog-badge]][changelog] [![release-badge]][release-link] [![Apache licensed]][Apache link]



## Run the block producer

Once you have completed the [pre-requisites](https://github.com/openmina/openmina/blob/main/docs/producer-demo.md#prerequisites) for your operating system, follow these steps:

1. **Clone this repository:**
    ```bash
    git clone https://github.com/openmina/openmina.git
    ```

2. **Navigate to the repository:**

    ```bash
    cd openmina
    ```

3. **Run the following command to start the demo:**
    ```sh
    docker compose -f docker-compose.local.producers.yml up
    ```

4. **Open you browser and visit http://localhost:8070**

![image](https://github.com/user-attachments/assets/c8929509-f68b-4281-bcb9-bad03029fa2f)


## Description


The Open Mina Node is a Mina node written completely in Rust and capable of verifying blocks of transactions, producing blocks and generating SNARKs.

In the design of the Open Mina node, we are utilizing much of the same logic as in the Mina Web Node. The key difference is that unlike the Web Node, which is an in-browser node with limited resources, the Open Mina node is able to perform resource-intensive tasks such as SNARK proof generation.




## Overview of the Node’s current functionalities


| Current functionalities | In Development | Future Plans |  
|---------------------------------------------------------------------------------------------|--------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|  
| &#9745; **Produce and prove blocks** (with and without transactions). | &#9744; Receiving and broadcasting transactions from/into the transaction pool. | &#9744; Direct transfer of MINA funds |  
| &#9745; **Produce SNARK proofs** for transactions. | &#9744; A block replayer that uses data from the archive nodes| &#9744; O1JS support for Rust node (ability to directly inject simple transactions).|  
| &#9745; **Connect to the network** and sync up to the best tip block | | |  
| &#9745; **Validate and apply new blocks** and transactions to update consensus and ledger state. | | |  
| &#9745; **Broadcast messages**: blocks, SNARK pool | | |

Please note that receiving and broadcasting transactions from/into the transaction pool is already possible, but is still an early alpha version and needs more work.


## Updates to the Front End

We've added two new pages to the node's front end:

### Mempool

![image](https://github.com/user-attachments/assets/a66b993d-5a9f-42a7-a946-f19f6e18e6ab)


Shows a list of the transactions from the pool and a side panel detail.

### Benchmarks

![image](https://github.com/user-attachments/assets/5aa9f0b8-2f53-4c2e-8b60-ed2ccaa7335b)

The benchmarks page helps us to send transactions. The transactions are signed in the front end by the Mina signer.
Every user can send transactions and they can see in the mempool whether the transactions were sent by their node. 

## Launch the block producer demo

Run the Open Mina block producer node by following this [guide](https://github.com/openmina/openmina/blob/main/docs/producer-demo.md).


## How to launch the node (with Docker compose):

Run:

```
docker compose up
```

Then visit http://localhost:8070 in your browser.

![image](https://github.com/user-attachments/assets/b8e10a12-ec96-44a9-951a-ef0c1b291428)


By default, `docker compose up` will use the latest node and frontend images available (tagged with `latest`), but specific versions can be selected by using the `OPENMINA_TAG` and `OPENMINA_FRONTEND_TAG` variables.

## How to launch the node (without Docker compose):

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

**Pre-requisites:**

Ubuntu or Debian-based Linux distribution with the following packages installed:

- `curl`
- `git`
- `libssl-dev`
- `pkg-config`
- `protobuf-compiler`
- `build-essential`

Example (debian-based):

``` sh
# Either using "sudo" or as the "root" user
sudo apt install curl git libssl-dev pkg-config protobuf-compiler build-essential
```

Example (macOS):

If you have not yet installed homebrew:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

``` sh
brew install curl git openssl pkg-config protobuf gcc make
```

**Steps (for Debian-based Linux distros and macOS):**

Open up the command line and enter the following:

And then:

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

## How to launch the UI:

## Prerequisites

### 1. Node.js v20.11.1

#### MacOS
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node@20.11.1
```

#### Linux
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
source ~/.bashrc
nvm install 20.11.1
```

#### Windows
Download [Node.js v20.11.1](https://nodejs.org/) from the official website, open the installer and follow the prompts to complete the installation.

### 2. Angular CLI v16.2.0
```bash
npm install -g @angular/cli@16.2.0
```

### 3. Installation
Open a terminal and navigate to this project's root directory

```bash
cd PROJECT_LOCATION/openmina/frontend
```
Install the dependencies
```bash
npm install
```

## Run the application

```bash
npm start
```

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

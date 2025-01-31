# How to build and launch a node from source

This installation guide has been tested on Debian and Ubuntu and should work on most distributions of Linux.

## Prerequisites

Ubuntu or Debian-based Linux distribution with the following packages installed:

- `curl`
- `git`
- `libssl-dev`
- `pkg-config`
- `protobuf-compiler`
- `build-essential`

Example (debian-based):

```sh
# Either using "sudo" or as the "root" user
sudo apt install curl git libssl-dev pkg-config protobuf-compiler build-essential
```

Example (macOS):

If you have not yet installed homebrew:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

```sh
brew install curl git openssl pkg-config protobuf gcc make
```

## Steps (for Debian-based Linux distros and macOS):

Open up the command line and enter the following:

And then:

```sh
# Install rustup and set the default Rust toolchain to 1.84 (newer versions work too)
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.84
# Setup the current shell with rustup
source "$HOME/.cargo/env"
# Clone the openmina repository
git clone https://github.com/openmina/openmina.git
cd openmina/
# Build and run the node
cargo run --release -p cli node
```

# How to launch the UI:

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

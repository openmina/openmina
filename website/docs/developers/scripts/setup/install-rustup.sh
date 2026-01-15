# Install rustup with default stable toolchain
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.92

# Setup the current shell
source "$HOME/.cargo/env"
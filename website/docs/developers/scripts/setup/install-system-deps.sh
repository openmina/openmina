sudo apt update
sudo apt install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    git \
    curl \
    shellcheck
    curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

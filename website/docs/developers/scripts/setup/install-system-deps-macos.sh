# Install Homebrew if not already installed
if ! command -v brew &> /dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install system dependencies
brew install \
    openssl \
    pkg-config \
    protobuf \
    git \
    curl \
    shellcheck

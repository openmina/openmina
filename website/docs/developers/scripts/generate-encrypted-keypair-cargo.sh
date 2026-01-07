#!/bin/bash

set -eou pipefail

# This creates `web-node-key.zip` containing:
#
# - `producer-key` (encrypted private key)
# - `producer-key.pub` (public key)
# - `producer-key-password` (password file)

# Create temporary working directory
mkdir producer-key-tmp

# Create password file
echo "mypassword" > producer-key-tmp/producer-key-password

# Generate encrypted key pair
cargo run -r --bin mina -- misc mina-encrypted-key \
  --file producer-key-tmp/producer-key \
  "$(cat producer-key-tmp/producer-key-password)"

# Package into ZIP for upload through the web interface and clean up intermediates
cd producer-key-tmp
zip web-node-key.zip producer-key producer-key.pub producer-key-password
mv web-node-key.zip ../
cd ../
rm -rf producer-key-tmp

#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Error: VERSION is required. Usage: $0 <version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

VERSION="$1"

echo "Updating version to $VERSION in workspace Cargo.toml..."

# Update version in workspace package section of root Cargo.toml
# All member crates inherit version via `version.workspace = true`
sed -i.bak 's/^version = "[^"]*"/version = "'"$VERSION"'"/' ./Cargo.toml

# Clean up backup file
rm -f ./Cargo.toml.bak

echo "Version updated to $VERSION in workspace Cargo.toml"
echo "All member crates inherit this version via workspace inheritance."

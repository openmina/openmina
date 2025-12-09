#!/bin/bash
set -e

echo "Validating codebase for release..."

echo "Setting up required tools..."
make setup-taplo

echo "Cleaning build artifacts to avoid version conflicts..."
cargo clean

echo "=== Testing stable Rust packages ==="

echo "Testing mina-node-native..."
make test-node-native

echo "Testing p2p..."
make test-p2p

echo "Testing account..."
make test-account

echo "=== Cleaning for nightly Rust packages ==="
cargo clean

echo "Testing ledger..."
make test-ledger

echo "Testing mina-p2p-messages..."
make test-p2p-messages

echo "Testing vrf..."
make test-vrf

echo "Checking code formatting..."
make check-format

echo "Fixing trailing whitespace..."
make fix-trailing-whitespace

echo "Verifying no trailing whitespace remains..."
make check-trailing-whitespace

echo "Release validation completed successfully!"

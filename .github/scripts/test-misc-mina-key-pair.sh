#!/bin/bash

set -euo pipefail

# Test the mina misc mina-key-pair command

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

MINA_BIN="${MINA_BIN:-./target/release/mina}"

echo "Testing: mina misc mina-key-pair"
echo ""

# Test 1: Basic mina-key-pair command
echo "Test 1: mina misc mina-key-pair (basic)"
if "$MINA_BIN" misc mina-key-pair > /dev/null 2>&1; then
    echo "✓ Command executed successfully"
else
    echo "✗ Test failed: Command failed to execute"
    exit 1
fi
echo ""

echo ""
echo "✓ All tests passed!"

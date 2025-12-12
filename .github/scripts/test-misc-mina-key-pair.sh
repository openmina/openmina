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

# Test 2: mina-key-pair with --web-node-secrets flag
echo "Test 2: mina misc mina-key-pair --web-node-secrets"
OUTPUT=$("$MINA_BIN" misc mina-key-pair --web-node-secrets)
EXIT_STATUS=$?

# Verify command executed successfully
if [ $EXIT_STATUS -ne 0 ]; then
    echo "✗ Test failed: Command failed to execute"
    exit 1
fi
echo "✓ Command executed successfully"

# Verify output is valid JSON
if ! echo "$OUTPUT" | jq empty 2>/dev/null; then
    echo "✗ Test failed: Output is not valid JSON"
    echo "Output was: $OUTPUT"
    exit 1
fi
echo "✓ Output is valid JSON"

# Verify JSON contains "publicKey" field with a string value
PUBLIC_KEY=$(echo "$OUTPUT" | jq -r '.publicKey' 2>/dev/null)
if [ -z "$PUBLIC_KEY" ] || [ "$PUBLIC_KEY" = "null" ]; then
    echo "✗ Test failed: JSON does not contain 'publicKey' field with a string value"
    echo "Output was: $OUTPUT"
    exit 1
fi
echo "✓ JSON contains 'publicKey' field with value: $PUBLIC_KEY"

# Verify JSON contains "privateKey" field with a string value
PRIVATE_KEY=$(echo "$OUTPUT" | jq -r '.privateKey' 2>/dev/null)
if [ -z "$PRIVATE_KEY" ] || [ "$PRIVATE_KEY" = "null" ]; then
    echo "✗ Test failed: JSON does not contain 'privateKey' field with a string value"
    echo "Output was: $OUTPUT"
    exit 1
fi
echo "✓ JSON contains 'privateKey' field with value: [REDACTED]"

echo ""
echo "✓ All tests passed!"

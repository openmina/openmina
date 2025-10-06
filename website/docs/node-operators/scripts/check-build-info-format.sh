#!/bin/bash
# Verify that build-info output format matches the documented example

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_FILE="$SCRIPT_DIR/../example-output/build-info-example.txt"

# Get actual build-info output
echo "Getting build-info output..."
ACTUAL_OUTPUT=$(docker run --rm o1labs/mina-rust:latest build-info)

# Extract field names from example file (everything before the colon)
echo "Checking that all expected fields are present..."
EXPECTED_FIELDS=(
  "Version"
  "Build time"
  "Commit SHA"
  "Commit time"
  "Commit branch"
  "Rustc channel"
  "Rustc version"
)

# Check that each expected field exists in the actual output
MISSING_FIELDS=()
for field in "${EXPECTED_FIELDS[@]}"; do
  if ! echo "$ACTUAL_OUTPUT" | grep -q "^${field}:"; then
    MISSING_FIELDS+=("$field")
  fi
done

if [ ${#MISSING_FIELDS[@]} -gt 0 ]; then
  echo "ERROR: Missing fields in build-info output:"
  printf '  - %s\n' "${MISSING_FIELDS[@]}"
  echo ""
  echo "Expected fields:"
  printf '  - %s\n' "${EXPECTED_FIELDS[@]}"
  echo ""
  echo "Actual output:"
  echo "$ACTUAL_OUTPUT"
  exit 1
fi

# Verify field order matches the example
echo "Checking that field order matches the example..."
EXAMPLE_ORDER=$(grep -o '^[^:]*:' "$EXAMPLE_FILE")
ACTUAL_ORDER=$(echo "$ACTUAL_OUTPUT" | grep -o '^[^:]*:')

if [ "$EXAMPLE_ORDER" != "$ACTUAL_ORDER" ]; then
  echo "ERROR: Field order doesn't match the example"
  echo ""
  echo "Expected order:"
  echo "$EXAMPLE_ORDER"
  echo ""
  echo "Actual order:"
  echo "$ACTUAL_ORDER"
  exit 1
fi

echo "✓ Build-info format matches the documented example"
echo ""
echo "Actual output:"
echo "$ACTUAL_OUTPUT"

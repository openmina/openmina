#!/usr/bin/env bash

set -euo pipefail

MINA_BIN="${MINA_BIN:-./target/release/mina}"
# Convert to absolute path if it's a relative path
if [[ "${MINA_BIN}" != /* ]]; then
  MINA_BIN="$(pwd)/${MINA_BIN}"
fi

# Add the directory containing the mina binary to PATH
# so documentation scripts can find 'mina' command
MINA_DIR="$(dirname "${MINA_BIN}")"
export PATH="${MINA_DIR}:${PATH}"

SCRIPT_DIR="website/docs/developers/scripts/cli"
QUERY_FILE="website/docs/developers/api-and-data/scripts/graphql-api/queries/query/sync-status.graphql"
GRAPHQL_NODE="${GRAPHQL_NODE:-https://mina-rust-plain-3.gcp.o1test.net/graphql}"

echo "Testing that 'mina internal graphql' commands are available..."

# Test 'mina internal' command exists
"${MINA_BIN}" internal --help > /dev/null || {
  echo "ERROR: 'mina internal' command not found"
  exit 1
}

# Test 'mina internal graphql' command exists
"${MINA_BIN}" internal graphql --help > /dev/null || {
  echo "ERROR: 'mina internal graphql' command not found"
  exit 1
}

# Test 'mina internal graphql list' command exists
"${MINA_BIN}" internal graphql list --help > /dev/null || {
  echo "ERROR: 'mina internal graphql list' command not found"
  exit 1
}

# Test 'mina internal graphql inspect' command exists
"${MINA_BIN}" internal graphql inspect --help > /dev/null || {
  echo "ERROR: 'mina internal graphql inspect' command not found"
  exit 1
}

# Test 'mina internal graphql run' command exists
"${MINA_BIN}" internal graphql run --help > /dev/null || {
  echo "ERROR: 'mina internal graphql run' command not found"
  exit 1
}

echo ""
echo "Testing documentation scripts execution against ${GRAPHQL_NODE}..."

# Function to test a script against the o1Labs node
test_script() {
  local script="$1"
  local extra_args="${2:-}"

  if [[ ! -f "${SCRIPT_DIR}/${script}" ]]; then
    echo "ERROR: Script ${script} not found"
    exit 1
  fi

  echo "Running ${script}..."

  # Export MINA_BIN for scripts to use, and add --node flag if needed
  export MINA_BIN
  if [[ -n "${extra_args}" ]]; then
    # Run script with modifications
    if output=$(bash -c "cd ${SCRIPT_DIR} && ${extra_args}" 2>&1); then
      echo "  SUCCESS: ${script} executed successfully"
      return 0
    else
      echo "ERROR: ${script} failed:"
      echo "$output"
      exit 1
    fi
  else
    # Run script as-is
    if output=$(bash "${SCRIPT_DIR}/${script}" 2>&1); then
      echo "  SUCCESS: ${script} executed successfully"
      return 0
    else
      echo "ERROR: ${script} failed:"
      echo "$output"
      exit 1
    fi
  fi
}

# Test all documentation scripts against the o1Labs node
# Scripts that don't have --node need it added
test_script "graphql-list.sh" "${MINA_BIN} internal graphql list --node ${GRAPHQL_NODE}"
test_script "graphql-inspect.sh" "${MINA_BIN} internal graphql inspect syncStatus --node ${GRAPHQL_NODE}"
test_script "graphql-inspect-remote.sh"
test_script "graphql-run-simple.sh" "${MINA_BIN} internal graphql run 'query { syncStatus }' --node ${GRAPHQL_NODE}"
test_script "graphql-run-stdin.sh" "echo 'query { version }' | ${MINA_BIN} internal graphql run --node ${GRAPHQL_NODE}"

# graphql-run-file.sh needs a query.graphql file in the current directory
echo "Running graphql-run-file.sh..."
if [[ ! -f "${QUERY_FILE}" ]]; then
  echo "ERROR: Query file ${QUERY_FILE} not found"
  exit 1
fi
# Create symlink in current directory for the script
ln -sf "${PWD}/${QUERY_FILE}" query.graphql
if output=$(${MINA_BIN} internal graphql run -f query.graphql --node "${GRAPHQL_NODE}" 2>&1); then
  echo "  SUCCESS: graphql-run-file.sh executed successfully"
else
  echo "ERROR: graphql-run-file.sh failed:"
  echo "$output"
  rm -f query.graphql
  exit 1
fi
rm -f query.graphql

test_script "graphql-run-variables.sh" "${MINA_BIN} internal graphql run 'query(\$maxLen: Int!) { bestChain(maxLength: \$maxLen) { stateHash } }' -v '{\"maxLen\": 5}' --node ${GRAPHQL_NODE}"
test_script "graphql-run-remote.sh"

echo ""
echo "SUCCESS: All 'mina internal graphql' commands and documentation scripts tested"

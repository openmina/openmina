#!/bin/bash

# Test GraphQL command scripts
# This corresponds to the second step in test-docs-graphql-api.yaml
#
# Usage:
#   ./test-graphql-command-scripts.sh
#
# This script tests GraphQL API command scripts by executing them and validating responses

set -e

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "jq is required but not installed."
    echo "Install it with:"
    echo "  - Ubuntu/Debian: sudo apt-get install jq"
    echo "  - macOS: brew install jq"
    exit 1
fi

echo "🔍 Testing GraphQL API command scripts..."

# Dynamically discover all bash scripts in the queries/curl directory (only test queries, not mutations)
script_dir="website/docs/developers/api-and-data/scripts/graphql-api/queries/curl"

if [ ! -d "$script_dir" ]; then
  echo "❌ Script directory not found: $script_dir"
  exit 1
fi

# Get all .sh files in the curl directory
script_files=$(ls "$script_dir"/*.sh 2>/dev/null)

if [ -z "$script_files" ]; then
  echo "❌ No bash scripts found in $script_dir"
  exit 1
fi

failed=0

for script_file in $script_files; do
  if [ ! -f "$script_file" ]; then
    echo "❌ Script file not found: $script_file"
    failed=$((failed + 1))
    continue
  fi

  echo "Testing script: $script_file"

  # Execute the script and capture output
  if output=$(bash "$script_file" 2>&1); then
    echo "✅ Script executed successfully"

    # Try to parse output as JSON using jq
    if echo "$output" | jq . > /dev/null 2>&1; then
      # Valid JSON response - check for GraphQL errors
      if echo "$output" | jq -e '.errors' > /dev/null 2>&1; then
        echo "❌ Script returned GraphQL errors:"
        echo "$output" | jq '.errors'
        failed=$((failed + 1))
      elif echo "$output" | jq -e '.data' > /dev/null 2>&1; then
        echo "✅ Script response contains valid data: $(echo "$output" | head -c 100)..."
      else
        echo "⚠️  Unexpected JSON response format: $(echo "$output" | head -c 100)..."
      fi
    else
      echo "❌ Script did not return valid JSON: $(echo "$output" | head -c 100)..."
      failed=$((failed + 1))
    fi
  else
    echo "❌ Script execution failed: $script_file"
    failed=$((failed + 1))
  fi

  echo "---"
done

if [ $failed -gt 0 ]; then
  echo "💥 $failed GraphQL script tests failed"
  echo "Some GraphQL API scripts may need updates."
  exit 1
else
  echo "🎉 All GraphQL API command scripts are working"
fi
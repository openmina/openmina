#!/bin/bash

# Test Documentation Scripts - GraphQL API (Local Runner)
# This script contains the same logic as .github/workflows/test-docs-graphql-api.yaml
#
# Usage:
#   ./test-graphql-api-local.sh
#
# Or run individual steps:
#   ./.github/scripts/test-script-query-consistency.sh
#   ./.github/scripts/test-graphql-command-scripts.sh
#
# This script runs all GraphQL API tests locally without GitHub Actions

set -e

echo "Testing GraphQL API Scripts Locally"
echo "===================================="

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "❌ jq is required but not installed."
    echo "Install it with:"
    echo "  - Ubuntu/Debian: sudo apt-get install jq"
    echo "  - macOS: brew install jq"
    exit 1
fi

echo "✅ jq is installed: $(jq --version)"
echo ""

# Test 1: Script and query file consistency
echo "🔍 Testing consistency between bash scripts and GraphQL query files..."

script_dir="website/docs/developers/api-and-data/scripts/graphql-api/queries/curl"
query_dir="website/docs/developers/api-and-data/scripts/graphql-api/queries/query"

inconsistent=0

# Check that each script that references a query file has a corresponding .graphql file
for script_file in "$script_dir"/*.sh; do
  if [ ! -f "$script_file" ]; then
    continue
  fi

  script_name=$(basename "$script_file" .sh)

  # Look for query file references in the script
  if grep -q "query/" "$script_file"; then
    expected_query_file="$query_dir/$script_name.graphql"

    if [ -f "$expected_query_file" ]; then
      echo "✅ $script_name has corresponding query file"
    else
      echo "❌ $script_name missing query file: $expected_query_file"
      inconsistent=$((inconsistent + 1))
    fi
  fi
done

# Check that each query file has a corresponding script
for query_file in "$query_dir"/*.graphql; do
  if [ ! -f "$query_file" ]; then
    continue
  fi

  query_name=$(basename "$query_file" .graphql)
  expected_script_file="$script_dir/$query_name.sh"

  if [ -f "$expected_script_file" ]; then
    echo "✅ $query_name has corresponding script file"
  else
    echo "❌ $query_name query file has no corresponding script: $expected_script_file"
    inconsistent=$((inconsistent + 1))
  fi
done

if [ $inconsistent -gt 0 ]; then
  echo "$inconsistent script/query file inconsistencies found"
  exit 1
else
  echo "All scripts and query files are consistent"
fi

echo ""

# Test 2: GraphQL command scripts
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
    echo "✅ Script executed successfully with output $output"

    # Try to parse output as JSON using jq
    if json_response=$(echo "$output" | jq . 2>/dev/null); then
      # Valid JSON response - check for GraphQL errors
      if echo "$json_response" | jq -e '.errors' > /dev/null 2>&1; then
        echo "❌ Script returned GraphQL errors:"
        echo "$json_response" | jq '.errors'
        failed=$((failed + 1))
      elif echo "$json_response" | jq -e '.data' > /dev/null 2>&1; then
        echo "✅ Script response contains valid data: $(echo "$json_response" | head -c 100)..."
      else
        echo "⚠️  Unexpected JSON response format: $(echo "$json_response" | head -c 100)..."
      fi
    else
      echo "❌ Script did not return valid JSON"
      failed=$((failed + 1))
    fi
  else
    echo "❌ Script execution failed: $script_file"
    failed=$((failed + 1))
  fi

  echo "---"
done

if [ $failed -gt 0 ]; then
  echo "$failed GraphQL script tests failed"
  echo "Some GraphQL API scripts may need updates."
  exit 1
else
  echo "All GraphQL API command scripts are working"
fi

echo ""
echo "All tests passed successfully!"
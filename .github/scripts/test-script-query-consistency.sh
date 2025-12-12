#!/bin/bash

# Test script and query file consistency
# This corresponds to the first step in test-docs-graphql-api.yaml
#
# Usage:
#   ./test-script-query-consistency.sh
#
# This script tests consistency between bash scripts and GraphQL query files

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
  echo "💥 $inconsistent script/query file inconsistencies found"
  exit 1
else
  echo "🎉 All scripts and query files are consistent"
fi
#!/bin/bash

# Test block producer node GraphQL endpoints
# This corresponds to the test-docs-infrastructure.yaml workflow
#
# Usage:
#   ./test-block-producer-node-connectivity.sh
#
# This script tests GraphQL connectivity to o1Labs block producer nodes

echo "🔍 Testing block producer node GraphQL connectivity..."

# Read block producer nodes from file
bp_nodes_file="website/docs/developers/scripts/infrastructure/block-producer-nodes.txt"

if [ ! -f "$bp_nodes_file" ]; then
  echo "❌ Block producer nodes file not found: $bp_nodes_file"
  exit 1
fi

bp_nodes=$(cat "$bp_nodes_file")
failed=0

for node_url in $bp_nodes; do
  echo "Testing GraphQL endpoint: $node_url"

  # Test basic HTTP connectivity
  if curl -s --connect-timeout 10 --max-time 30 "$node_url" > /dev/null 2>&1; then
    echo "✅ $node_url is reachable via HTTP"
  else
    echo "❌ $node_url is not reachable via HTTP"
    failed=$((failed + 1))
    continue
  fi

  # Test GraphQL endpoint using website scripts
  graphql_url="${node_url}graphql"

  # Test daemon status query using the website script
  if response=$(bash website/docs/developers/scripts/graphql-api/queries/curl/daemon-status.sh "$graphql_url" 2>&1); then
    # Check if it's valid JSON
    if echo "$response" | jq . > /dev/null 2>&1; then
      # Check for GraphQL errors
      if echo "$response" | jq -e '.errors' > /dev/null 2>&1; then
        echo "⚠️  $graphql_url returned GraphQL error:"
        echo "$response" | jq '.errors'
      # Check for valid data
      elif echo "$response" | jq -e '.data.daemonStatus' > /dev/null 2>&1; then
        echo "✅ $graphql_url GraphQL query successful"
        sync_status=$(echo "$response" | jq -r '.data.daemonStatus.syncStatus // "unknown"')
        chain_id=$(echo "$response" | jq -r '.data.daemonStatus.chainId // "unknown"')
        echo "   Sync Status: $sync_status, Chain ID: ${chain_id:0:16}..."
      else
        echo "⚠️  $graphql_url unexpected response format"
      fi
    else
      echo "⚠️  $graphql_url did not return valid JSON: $(echo "$response" | head -c 100)..."
    fi
  else
    echo "❌ $graphql_url GraphQL query failed"
    failed=$((failed + 1))
  fi

  echo "---"
done

if [ $failed -gt 0 ]; then
  echo "💥 $failed block producer node tests failed"
  echo "Infrastructure issues detected. Please check block producer node status."
  exit 1
else
  echo "🎉 All block producer nodes are healthy and responding"
fi

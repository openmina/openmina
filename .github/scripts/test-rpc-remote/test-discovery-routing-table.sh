#!/bin/bash

# Test /discovery/routing_table endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /discovery/routing_table endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/discovery-routing-table.sh "$RPC_ENDPOINT")
if [ "$response" = "null" ]; then
  echo "Routing table: null (not available)"
elif echo "$response" | jq -e '.this_key' > /dev/null 2>&1; then
  bucket_count=$(echo "$response" | jq '.buckets | length')
  echo "Routing table: $bucket_count buckets"
else
  echo "Routing table: unexpected response"
  echo "Response: $response"
fi

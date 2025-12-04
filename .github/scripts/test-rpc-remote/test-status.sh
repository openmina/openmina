#!/bin/bash

# Test /status endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /status endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/status.sh "$RPC_ENDPOINT")
echo "Response: $response"
chain_id=$(echo "$response" | jq -r '.chain_id // empty')
if [ -n "$chain_id" ]; then
  echo "Status: chain_id = $chain_id"
else
  echo "Status: FAILED - no chain_id in response"
  exit 1
fi

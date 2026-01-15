#!/bin/bash

# Test /scan-state/summary endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /scan-state/summary endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/scan-state-summary.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e '.block' > /dev/null 2>&1; then
  block_height=$(echo "$response" | jq -r '.block.height')
  echo "Scan state: block height = $block_height"
else
  echo "Scan state: FAILED - no block in response"
  echo "Response: $response"
  exit 1
fi

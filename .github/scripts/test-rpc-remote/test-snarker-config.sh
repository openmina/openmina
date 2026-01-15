#!/bin/bash

# Test /snarker/config endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /snarker/config endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/snarker-config.sh "$RPC_ENDPOINT")
if [ "$response" = "null" ]; then
  echo "SNARK config: null (no snarker configured)"
elif echo "$response" | jq -e '.public_key' > /dev/null 2>&1; then
  echo "SNARK config: snarker configured"
else
  echo "SNARK config: unexpected response"
  echo "Response: $response"
fi

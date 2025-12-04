#!/bin/bash

# Test /stats/block_producer endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /stats/block_producer endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/stats-block-producer.sh "$RPC_ENDPOINT")
if [ "$response" = "null" ]; then
  echo "Stats/block_producer: null (no block producer stats available)"
elif echo "$response" | jq -e 'type == "object"' > /dev/null 2>&1; then
  echo "Stats/block_producer: OK"
else
  echo "Stats/block_producer: unexpected response type"
  echo "Response: $response"
fi

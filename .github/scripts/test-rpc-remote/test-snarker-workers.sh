#!/bin/bash

# Test /snarker/workers endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /snarker/workers endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/snarker-workers.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  worker_count=$(echo "$response" | jq 'length')
  echo "SNARK workers: $worker_count workers"
else
  echo "SNARK workers: FAILED - response is not an array"
  echo "Response: $response"
  exit 1
fi

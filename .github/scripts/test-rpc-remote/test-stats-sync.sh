#!/bin/bash

# Test /stats/sync endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /stats/sync endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/stats-sync.sh "$RPC_ENDPOINT")
if [ "$response" = "null" ]; then
  echo "Stats/sync: null (no sync stats available yet)"
elif echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  count=$(echo "$response" | jq 'length')
  echo "Stats/sync: $count sync snapshots"
else
  echo "Stats/sync: unexpected response type"
  echo "Response: $response"
fi

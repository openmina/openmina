#!/bin/bash

# Test /transaction-pool endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /transaction-pool endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/transaction-pool.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  tx_count=$(echo "$response" | jq 'length')
  echo "Transaction pool: $tx_count transactions"
else
  echo "Transaction pool: FAILED - response is not an array"
  echo "Response: $response"
  exit 1
fi

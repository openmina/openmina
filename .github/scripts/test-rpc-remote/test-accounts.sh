#!/bin/bash

# Test /accounts endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /accounts endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/accounts.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  account_count=$(echo "$response" | jq 'length')
  echo "Accounts: $account_count accounts"
else
  echo "Accounts: FAILED - response is not an array"
  echo "Response: $response"
  exit 1
fi

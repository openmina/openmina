#!/bin/bash

# Test /state endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /state endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/state.sh "$RPC_ENDPOINT")
if [ -n "$response" ] && [ "$response" != "null" ]; then
  echo "State: OK (response received)"
else
  echo "State: empty or null response"
fi

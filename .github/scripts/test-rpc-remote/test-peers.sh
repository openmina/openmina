#!/bin/bash

# Test /state/peers endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /state/peers endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/peers.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  peer_count=$(echo "$response" | jq 'length')
  echo "Peers: found $peer_count peers"
else
  echo "Peers: FAILED - response is not an array"
  echo "Response: $response"
  exit 1
fi

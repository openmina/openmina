#!/bin/bash

# Test /state/message-progress endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /state/message-progress endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/message-progress.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e '.messages_stats' > /dev/null 2>&1; then
  echo "Message progress: OK"
else
  echo "Message progress: FAILED - unexpected response"
  echo "Response: $response"
  exit 1
fi

#!/bin/bash

# Test /build_env endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /build_env endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/build-env.sh "$RPC_ENDPOINT")
echo "Response: $response"
if echo "$response" | jq -e '.git.commit_hash' > /dev/null 2>&1; then
  commit_hash=$(echo "$response" | jq -r '.git.commit_hash')
  echo "Build env: commit_hash = $commit_hash"
else
  echo "Build env: FAILED - no git.commit_hash in response"
  exit 1
fi

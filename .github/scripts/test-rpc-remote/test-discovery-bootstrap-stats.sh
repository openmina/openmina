#!/bin/bash

# Test /discovery/bootstrap_stats endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /discovery/bootstrap_stats endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/discovery-bootstrap-stats.sh "$RPC_ENDPOINT")
if [ "$response" = "null" ]; then
  echo "Bootstrap stats: null (bootstrap complete or not started)"
else
  echo "Bootstrap stats: available"
  echo "Response: $response"
fi

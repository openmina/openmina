#!/bin/bash

# Test /snark-pool/jobs endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /snark-pool/jobs endpoint..."
response=$(website/docs/developers/api-and-data/scripts/rpc-api/curl/snark-pool-jobs.sh "$RPC_ENDPOINT")
if echo "$response" | jq -e 'type == "array"' > /dev/null 2>&1; then
  job_count=$(echo "$response" | jq 'length')
  echo "SNARK pool: $job_count jobs"
else
  echo "SNARK pool: FAILED - response is not an array"
  echo "Response: $response"
  exit 1
fi

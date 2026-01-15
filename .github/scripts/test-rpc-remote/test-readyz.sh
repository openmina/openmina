#!/bin/bash

# Test /readyz endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /readyz endpoint..."
response=$(curl -s --max-time 10 -w "%{http_code}" "$RPC_ENDPOINT/readyz" -o /dev/null)
if [ "$response" = "200" ]; then
  echo "Readyz: OK (HTTP $response)"
else
  echo "Readyz: FAILED (HTTP $response)"
  echo "Warning: Node may not be ready yet"
fi

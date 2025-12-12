#!/bin/bash

# Test /healthz endpoint
# Usage: $0 [RPC_ENDPOINT]

set -e

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

echo "Testing /healthz endpoint..."
response=$(curl -s --max-time 10 -w "%{http_code}" "$RPC_ENDPOINT/healthz" -o /dev/null)
if [ "$response" = "200" ]; then
  echo "Healthz: OK (HTTP $response)"
else
  echo "Healthz: FAILED (HTTP $response)"
  exit 1
fi

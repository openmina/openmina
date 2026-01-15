#!/bin/bash
# Usage: $0 [RPC_ENDPOINT] [FILTER]
# RPC_ENDPOINT: RPC endpoint URL (default: http://mina-rust-plain-3.gcp.o1test.net)
# FILTER: Optional JSONPath filter (e.g., "$.p2p")

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"
FILTER="${2:-}"

if [ -n "$FILTER" ]; then
  curl -s -X GET "$RPC_ENDPOINT/state?filter=$FILTER" \
    -H "Content-Type: application/json"
else
  curl -s -X GET "$RPC_ENDPOINT/state" \
    -H "Content-Type: application/json"
fi

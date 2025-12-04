#!/bin/bash
# Usage: $0 [RPC_ENDPOINT]
# RPC_ENDPOINT: RPC endpoint URL (default: http://mina-rust-plain-3.gcp.o1test.net)

RPC_ENDPOINT="${1:-http://mina-rust-plain-3.gcp.o1test.net}"

curl -s -X GET "$RPC_ENDPOINT/stats/sync" \
  -H "Content-Type: application/json"

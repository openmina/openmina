#!/usr/bin/env bash
# Query recent blocks from the archive node

ARCHIVE_NODE="${1:-https://devnet-archive-node-api.gcp.o1test.net}"

curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ blocks(limit: 5) { blockHeight stateHash creatorAccount { publicKey } } }"
  }' \
  "${ARCHIVE_NODE}/graphql" | jq .

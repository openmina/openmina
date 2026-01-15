#!/bin/bash
# Usage: $0 [GRAPHQL_ENDPOINT]
# GRAPHQL_ENDPOINT: GraphQL endpoint URL (default: http://mina-rust-plain-1.gcp.o1test.net/graphql)

GRAPHQL_ENDPOINT="${1:-http://mina-rust-plain-1.gcp.o1test.net/graphql}"

# Replace with your own node endpoint: http://localhost:3000/graphql
# WARNING: This mutation modifies the blockchain state
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUERY=$(tr '\n' ' ' < "$SCRIPT_DIR/../query/send-zkapp.graphql" | sed 's/  */ /g')

# Example variables - replace with actual zkApp transaction data
VARIABLES='{
  "input": {
    "zkappCommand": {
      "feePayer": {
        "body": {
          "publicKey": "B62qmGtQ7kn6zbw4tAYomBJJri1gZSThfQZJaMG6eR3tyNP3RiCcEQZ",
          "fee": "10000000",
          "validUntil": null,
          "nonce": 0
        },
        "authorization": "..."
      },
      "accountUpdates": [],
      "memo": "zkApp transaction"
    }
  }
}'

curl -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$QUERY\", \"variables\": $VARIABLES}"
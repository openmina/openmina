#!/bin/bash
# Usage: $0 [GRAPHQL_ENDPOINT]
# GRAPHQL_ENDPOINT: GraphQL endpoint URL (default: http://mina-rust-plain-1.gcp.o1test.net/graphql)

GRAPHQL_ENDPOINT="${1:-http://mina-rust-plain-1.gcp.o1test.net/graphql}"

# Replace with your own node endpoint: http://localhost:3000/graphql
# WARNING: This mutation modifies the blockchain state
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUERY=$(tr '\n' ' ' < "$SCRIPT_DIR/../query/send-delegation.graphql" | sed 's/  */ /g')

# Example variables - replace with actual values
VARIABLES='{
  "input": {
    "from": "B62qmGtQ7kn6zbw4tAYomBJJri1gZSThfQZJaMG6eR3tyNP3RiCcEQZ",
    "to": "B62qrPN5Y5yq8kGE3FbVKbGTdTAJNdtNtB5sNVpxyRwWGcDEhpMzc8g",
    "fee": "10000000",
    "memo": "Test delegation"
  },
  "signature": {
    "field": "...",
    "scalar": "..."
  }
}'

curl -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$QUERY\", \"variables\": $VARIABLES}"

#!/bin/bash
# Usage: $0 GRAPHQL_ENDPOINT
# GRAPHQL_ENDPOINT: GraphQL endpoint URL (required, must end with /graphql)
# Example: https://mina-rust-bp-1.gcp.o1test.net/graphql

if [ -z "$1" ]; then
    echo "Error: GRAPHQL_ENDPOINT is required"
    echo "Usage: $0 GRAPHQL_ENDPOINT"
    exit 1
fi

GRAPHQL_ENDPOINT="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Read the query and create JSON payload using jq for proper escaping
QUERY=$(< "$SCRIPT_DIR/queries/best-chain-transactions.graphql")
JSON_PAYLOAD=$(echo "{}" | jq --arg query "$QUERY" '.query = $query')
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD"

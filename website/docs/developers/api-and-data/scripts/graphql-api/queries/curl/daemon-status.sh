#!/bin/bash
# Usage: $0 [GRAPHQL_ENDPOINT]
# GRAPHQL_ENDPOINT: GraphQL endpoint URL (default: http://mina-rust-plain-1.gcp.o1test.net/graphql)

GRAPHQL_ENDPOINT="${1:-http://mina-rust-plain-1.gcp.o1test.net/graphql}"

# Replace with your own node endpoint: http://localhost:3000/graphql
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Read the query and create JSON payload using jq for proper escaping
QUERY=$(< "$SCRIPT_DIR/../query/daemon-status.graphql")
JSON_PAYLOAD=$(echo "{}" | jq --arg query "$QUERY" '.query = $query')
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD"
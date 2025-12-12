#!/bin/bash
# Usage: $0 [GRAPHQL_ENDPOINT]
# GRAPHQL_ENDPOINT: GraphQL endpoint URL (default: http://mina-rust-plain-1.gcp.o1test.net/graphql)

GRAPHQL_ENDPOINT="${1:-http://mina-rust-plain-1.gcp.o1test.net/graphql}"

# Replace with your own node endpoint: http://localhost:3000/graphql
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUERY=$(tr '\n' ' ' < "$SCRIPT_DIR/../query/transaction-status.graphql" | sed 's/  */ /g')
curl -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$QUERY\", \"variables\": {\"payment\": \"5Ju2GQkUHy8iMXaeJDbDCyaV8fR1in3ewuM82Rq4z3sdqNrKkzgp\"}}"

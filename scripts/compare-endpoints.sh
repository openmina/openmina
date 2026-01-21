#!/bin/bash
# Compare warp (3000) vs axum (3001) endpoint responses
# Usage: ./compare-endpoints.sh /endpoint [curl args...]
#
# Safe for large outputs - all operations on files, never loads body into bash vars.

set -e

if [[ -z "$1" ]]; then
    echo "Usage: $0 /endpoint [curl args...]"
    exit 1
fi

WARP_PORT=${WARP_PORT:-3000}
AXUM_PORT=${AXUM_PORT:-3001}
ENDPOINT="$1"
shift

# Temp files
WARP_HDR=$(mktemp)
WARP_BODY=$(mktemp)
AXUM_HDR=$(mktemp)
AXUM_BODY=$(mktemp)
trap 'rm -f "$WARP_HDR" "$WARP_BODY" "$AXUM_HDR" "$AXUM_BODY"' EXIT

# Fetch both (dump headers separately)
WARP_CODE=$(curl -sS -D "$WARP_HDR" -o "$WARP_BODY" -w "%{http_code}" "localhost:$WARP_PORT$ENDPOINT" "$@" 2>&1) || true
AXUM_CODE=$(curl -sS -D "$AXUM_HDR" -o "$AXUM_BODY" -w "%{http_code}" "localhost:$AXUM_PORT$ENDPOINT" "$@" 2>&1) || true

echo "=== $ENDPOINT ==="

# Status codes
echo "Status: warp=$WARP_CODE axum=$AXUM_CODE"
[[ "$WARP_CODE" != "$AXUM_CODE" ]] && echo "!! STATUS MISMATCH"

# File sizes (body only)
WARP_SIZE=$(wc -c < "$WARP_BODY" | tr -d ' ')
AXUM_SIZE=$(wc -c < "$AXUM_BODY" | tr -d ' ')
echo "Body size: warp=${WARP_SIZE}B axum=${AXUM_SIZE}B"

# Headers comparison (filter date/cors, operate on files)
echo "--- Headers ---"
grep -vi '^date:' "$WARP_HDR" | grep -vi '^access-control' | sort > "${WARP_HDR}.clean" || true
grep -vi '^date:' "$AXUM_HDR" | grep -vi '^access-control' | sort > "${AXUM_HDR}.clean" || true

if cmp -s "${WARP_HDR}.clean" "${AXUM_HDR}.clean"; then
    echo "(match, ignoring date/cors)"
else
    echo "Diff:"
    diff "${WARP_HDR}.clean" "${AXUM_HDR}.clean" | head -c 1000 || true
fi
rm -f "${WARP_HDR}.clean" "${AXUM_HDR}.clean"

# Body comparison
echo "--- Body ---"

# JSON structure check (jq reads file directly, output truncated)
if jq -e . "$WARP_BODY" >/dev/null 2>&1; then
    WARP_STRUCT=$(jq -r 'if type == "object" then keys | join(",") elif type == "array" then "array[\(length)]" else type end' "$WARP_BODY" 2>/dev/null | head -c 200)
    AXUM_STRUCT=$(jq -r 'if type == "object" then keys | join(",") elif type == "array" then "array[\(length)]" else type end' "$AXUM_BODY" 2>/dev/null | head -c 200)
    echo "Structure: warp=[$WARP_STRUCT] axum=[$AXUM_STRUCT]"
    [[ "$WARP_STRUCT" != "$AXUM_STRUCT" ]] && echo "!! STRUCTURE MISMATCH"
fi

# Binary comparison
if cmp -s "$WARP_BODY" "$AXUM_BODY"; then
    echo "(identical)"
else
    echo "(different)"
    # Show MD5 for large, sample for small
    if [[ $WARP_SIZE -gt 2000 || $AXUM_SIZE -gt 2000 ]]; then
        WARP_MD5=$(md5 -q "$WARP_BODY" 2>/dev/null || md5sum "$WARP_BODY" | cut -d' ' -f1)
        AXUM_MD5=$(md5 -q "$AXUM_BODY" 2>/dev/null || md5sum "$AXUM_BODY" | cut -d' ' -f1)
        echo "MD5: warp=$WARP_MD5 axum=$AXUM_MD5"
        echo "Sample (first 500B):"
        echo "warp: $(head -c 500 "$WARP_BODY")"
        echo "axum: $(head -c 500 "$AXUM_BODY")"
    else
        echo "Diff (truncated):"
        diff <(jq -S . "$WARP_BODY" 2>/dev/null || cat "$WARP_BODY") \
             <(jq -S . "$AXUM_BODY" 2>/dev/null || cat "$AXUM_BODY") | head -c 2000 || true
    fi
fi

#!/bin/sh

for id in "$@"; do
    curl -s "https://debug.dev.tezedge.com/message_hex/$id" | jq -r | xxd -r -p | tail -c +9 > "$id.bin"
done

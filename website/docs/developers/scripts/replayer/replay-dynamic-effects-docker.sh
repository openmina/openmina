#!/bin/bash
# Replay with dynamic effects using Docker

docker run --rm \
  -v "$(pwd)/mina-replay-test:/root/.mina" \
  -v "$(pwd)/my-effects.so:/effects/my-effects.so" \
  o1labs/mina-rust:latest \
  replay state-with-input-actions \
  --dir /root/.mina/recorder \
  --dynamic-effects-lib /effects/my-effects.so

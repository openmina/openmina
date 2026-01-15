#!/bin/bash
# Replay with build environment mismatch ignored using Docker

docker run --rm \
  -v "$(pwd)/mina-replay-test:/root/.mina" \
  o1labs/mina-rust:latest \
  replay state-with-input-actions \
  --dir /root/.mina/recorder \
  --ignore-build-env-mismatch

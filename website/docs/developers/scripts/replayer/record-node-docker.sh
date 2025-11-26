#!/bin/bash
# Record node execution using Docker

# Create directory for recording
mkdir -p mina-replay-test

# Run node with recording enabled using Docker
docker run --rm \
  -v "$(pwd)/mina-replay-test:/root/.mina" \
  o1labs/mina-rust:latest \
  node \
  --network devnet \
  --record state-with-input-actions \
  --work-dir /root/.mina

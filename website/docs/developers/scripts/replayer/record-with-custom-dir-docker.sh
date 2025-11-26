#!/bin/bash
# Record with custom directory using Docker

# Create custom directory for recording
mkdir -p my-custom-replay-dir

# Run node with custom recording directory
docker run --rm \
  -v "$(pwd)/my-custom-replay-dir:/root/.mina" \
  o1labs/mina-rust:latest \
  node \
  --network devnet \
  --record state-with-input-actions \
  --work-dir /root/.mina

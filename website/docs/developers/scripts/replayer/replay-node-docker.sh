#!/bin/bash
# Replay recorded node execution using Docker

# Replay from the recorded directory
docker run --rm \
  -v "$(pwd)/mina-replay-test:/root/.mina" \
  o1labs/mina-rust:latest \
  replay state-with-input-actions --dir /root/.mina/recorder

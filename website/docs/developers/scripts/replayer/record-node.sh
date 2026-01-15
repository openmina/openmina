#!/bin/bash
# Record node execution for debugging and replay

# Run node with recording enabled
mina node \
  --network devnet \
  --record state-with-input-actions \
  --work-dir ~/.mina-replay-test

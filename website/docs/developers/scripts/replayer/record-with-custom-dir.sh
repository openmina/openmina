#!/bin/bash
# Record node execution with custom working directory

WORK_DIR="$1"
if [ -z "$WORK_DIR" ]; then
  echo "Usage: $0 <work-directory>"
  exit 1
fi

# Run node with recording enabled using custom directory
mina node \
  --network devnet \
  --record state-with-input-actions \
  --work-dir "$WORK_DIR"

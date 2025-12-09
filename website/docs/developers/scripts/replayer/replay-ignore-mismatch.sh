#!/bin/bash
# Replay and ignore build environment differences

mina replay state-with-input-actions \
  --dir ~/.mina-replay-test/recorder \
  --ignore-mismatch

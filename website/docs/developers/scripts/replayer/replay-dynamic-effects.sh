#!/bin/bash
# Build custom effects library and replay with custom effects

# Build custom effects library
cargo build --release -p replay_dynamic_effects

# Replay with custom effects
mina replay state-with-input-actions \
  --dir ~/.mina-replay-test/recorder \
  --dynamic-effects-lib ./target/release/libreplay_dynamic_effects.so

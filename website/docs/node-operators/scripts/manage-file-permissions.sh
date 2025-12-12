#!/bin/bash
# Check file ownership
ls -la mina-workdir/

# Fix ownership for all files in mina-workdir
sudo chown -R "$(id -u)":"$(id -g)" mina-workdir/

# Set appropriate permissions for the producer key
chmod 600 mina-workdir/producer-key

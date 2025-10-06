#!/bin/bash
# Backup producer key (if running block producer)
cp -r mina-workdir/producer-key ~/mina-backup/

# Backup entire working directory
tar -czf "mina-backup-$(date +%Y%m%d).tar.gz" mina-workdir/

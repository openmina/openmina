#!/bin/bash
cat > .env << EOF
POSTGRES_PASSWORD=mina
PG_PORT=5432
PG_DB=archive
MINA_RUST_TAG=latest
EOF

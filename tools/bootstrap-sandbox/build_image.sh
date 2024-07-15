#!/usr/bin/env sh

docker build -t vladsimplestakingcom/bootstrap-rr:3.0.0-bullseye-devnet \
    -f tools/bootstrap-sandbox/Dockerfile target/record

# docker run --rm --name openmina-bootstrap-replayer -p 8303:8303 vladsimplestakingcom/bootstrap-rr:3.0.0-bullseye-devnet openmina-bootstrap-sandbox --listen='/ip4/0.0.0.0/tcp/8303' replay 327636

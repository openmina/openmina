#!/usr/bin/env sh

docker build -t vladsimplestakingcom/bootstrap-rr:2.0.0berkeley-rc1 \
    -f tools/bootstrap-sandbox/Dockerfile target/record

# docker run --rm --name openmina-bootstrap-replayer -p 8303:8303 vladsimplestakingcom/bootstrap-rr:2.0.0berkeley-rc1 openmina-bootstrap-sandbox --listen='/ip4/0.0.0.0/tcp/8303' replay 3830

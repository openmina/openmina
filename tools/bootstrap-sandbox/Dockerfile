FROM rust:1.76.0-bullseye as builder

RUN mkdir -p -m 0700 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo install --git https://github.com/openmina/openmina.git --branch feat/replace-libp2p-rebased-2 openmina-bootstrap-sandbox --locked

FROM debian:bullseye

RUN apt-get update && apt-get install -y libssl-dev

COPY --from=builder /usr/local/cargo/bin/openmina-bootstrap-sandbox \
    /usr/local/bin/openmina-bootstrap-sandbox

ENV MINA_RECORD_PATH=/tmp/default
COPY . /target/record

ENV RUST_LOG=info

WORKDIR /

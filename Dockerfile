ARG MINA_SNARK_WORKER_TAG=0.0.9

FROM rust:buster AS build
RUN rustup default 1.77 && rustup component add rustfmt
WORKDIR /openmina
COPY . .
RUN cargo build --release --package=cli --bin=openmina

FROM openmina/mina-snark-worker-prover:${MINA_SNARK_WORKER_TAG} as prover

FROM debian:buster
RUN apt-get update && apt-get install -y libjemalloc-dev libssl-dev libpq-dev curl jq procps
COPY --from=build /openmina/cli/bin/snark-worker /usr/local/bin/
COPY --from=build /openmina/target/release/openmina /usr/local/bin/
COPY --from=prover /usr/local/bin/mina /usr/local/bin
ENTRYPOINT [ "openmina" ]

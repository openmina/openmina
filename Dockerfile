ARG MINA_SNARK_WORKER_TAG=0.0.9

FROM rust:buster AS build
RUN apt-get update && apt-get install -y protobuf-compiler && apt-get clean
RUN rustup default 1.80 && rustup component add rustfmt
WORKDIR /openmina
COPY . .
RUN cargo build --release --package=cli --bin=openmina
RUN cargo build --release --features scenario-generators --bin openmina-node-testing

# necessary for proof generation when running a block producer.
RUN git clone --depth 1 https://github.com/openmina/circuit-blobs.git
RUN mv node/web/circuit-blobs/* ledger

FROM openmina/mina-snark-worker-prover:${MINA_SNARK_WORKER_TAG} AS prover

FROM debian:buster
RUN apt-get update && apt-get install -y libjemalloc2 libssl1.1 libpq5 curl jq procps && apt-get clean
COPY --from=build /openmina/cli/bin/snark-worker /usr/local/bin/
COPY --from=build /openmina/target/release/openmina /usr/local/bin/
COPY --from=build /openmina/target/release/openmina-node-testing /usr/local/bin/
COPY --from=prover /usr/local/bin/mina /usr/local/bin
ENTRYPOINT [ "openmina" ]

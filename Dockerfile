ARG MINA_SNARK_WORKER_TAG=0.0.9

FROM rust:buster AS build
RUN apt-get update && apt-get install -y protobuf-compiler && apt-get clean
RUN rustup default 1.79 && rustup component add rustfmt
WORKDIR /openmina
COPY . .
RUN cargo build --release --package=cli --bin=openmina
RUN cargo build --release --features scenario-generators --bin openmina-node-testing

FROM openmina/mina-snark-worker-prover:${MINA_SNARK_WORKER_TAG} as prover

FROM debian:buster
RUN apt-get update && apt-get install -y libjemalloc2 libssl1.1 libpq5 curl jq procps && apt-get clean
COPY --from=build /openmina/cli/bin/snark-worker /usr/local/bin/
COPY --from=build /openmina/target/release/openmina /usr/local/bin/
COPY --from=build /openmina/target/release/openmina-node-testing /usr/local/bin/
COPY --from=prover /usr/local/bin/mina /usr/local/bin
ENTRYPOINT [ "openmina" ]
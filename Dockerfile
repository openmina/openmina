FROM rust:bookworm AS build
RUN apt-get update && apt-get install -y protobuf-compiler build-essential cmake pkg-config libssl3 libssl-dev clang gcc make perl && apt-get clean
RUN rustup default 1.84 && rustup component add rustfmt
ENV OPENSSL_STATIC=1
WORKDIR /openmina
COPY . .
# Build with cache mount
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/openmina/target,id=rust-target \
    cargo build --release --package=cli --bin=openmina && \
    cp -r /openmina/target/release /openmina/release-bin/

# RUN --mount=type=cache,target=/usr/local/cargo/registry \
#     --mount=type=cache,target=/openmina/target,id=rust-target \
#     cargo build --release --features scenario-generators --bin openmina-node-testing && \
#     cp -r /openmina/target/release /openmina/testing-release-bin/

# necessary for proof generation when running a block producer.
RUN git clone --depth 1 https://github.com/openmina/circuit-blobs.git \
    && rm -rf circuit-blobs/berkeley_rc1 circuit-blobs/*/tests

FROM debian:bookworm
RUN apt-get update && apt-get install -y libjemalloc2 libssl3 libpq5 curl jq procps && apt-get clean

COPY --from=build /openmina/release-bin/openmina /usr/local/bin/
# COPY --from=build /openmina/testing-release-bin/openmina-node-testing /usr/local/bin/

RUN mkdir -p /usr/local/lib/openmina/circuit-blobs
COPY --from=build /openmina/circuit-blobs/ /usr/local/lib/openmina/circuit-blobs/

EXPOSE 3000
EXPOSE 8302

ENTRYPOINT [ "openmina" ]

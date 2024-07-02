FROM rust:1.79 as app-builder

WORKDIR /usr/src/openmina-producer-dashboard

COPY ../ .

RUN cd producer-dashboard && SQLX_OFFLINE=true cargo install --path .

FROM ubuntu:latest as mina-builder

RUN apt-get update && apt-get install -y openssl ca-certificates

# Build mina from source
RUN apt-get update && \
  apt-get -y --no-install-recommends install \
    libboost-dev \
    libboost-program-options-dev \
    libbz2-dev \
    libcap-dev \
    libffi-dev \
    libgflags-dev \
    libgmp-dev \
    libgmp3-dev \
    libjemalloc-dev \
    liblmdb-dev \
    liblmdb0 \
    libpq-dev \
    libprocps-dev \
    libsodium-dev \
    libssl-dev \
    build-essential \
    ca-certificates \
    capnproto \
    cmake \
    curl \
    file \
    git \
    unzip \
    rsync


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /go
RUN git clone https://github.com/MinaProtocol/mina.git

ENV DUNE_PROFILE=devnet

WORKDIR /go/mina
RUN git checkout 3.0.0devnet && \
    git submodule update --init --recursive && \
    git config --local --add submodule.recurse true

# RUN make libp2p_helper

RUN curl -s -L https://github.com/ocaml/opam/releases/download/2.1.2/opam-2.1.2-x86_64-linux -o /usr/local/bin/opam && chmod +x /usr/local/bin/opam

RUN apt-get -y --no-install-recommends install m4 pkg-config

RUN opam init --disable-sandboxing \
  && opam switch create . \
  && eval $(opam config env) \
  && opam switch import -y opam.export \
  && ./scripts/pin-external-packages.sh

RUN curl -L https://go.dev/dl/go1.19.linux-amd64.tar.gz -o go1.19.tar.gz \
    && tar -C /usr/local -xzf go1.19.tar.gz \
    && rm go1.19.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
RUN make libp2p_helper

ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt-get -y --no-install-recommends install zlib1g-dev
RUN eval $(opam config env) && make build_all_sigs
# RUN /bin/bash -c "source ~/.cargo/env && eval $(opam config env) && make build_all_sigs"

FROM ubuntu:latest

RUN apt-get update && apt-get install -y libpq5 libjemalloc2
  
COPY --from=app-builder /usr/local/cargo/bin/openmina-producer-dashboard /usr/local/bin/openmina-producer-dashboard
COPY --from=mina-builder /go/mina/src/app/libp2p_helper/result/bin/libp2p_helper /usr/local/bin/coda-libp2p_helper
COPY --from=mina-builder /go/mina/_build/default/src/app/cli/src/mina_testnet_signatures.exe /usr/local/bin/mina

# TODO: replace
ENTRYPOINT [ "openmina-producer-dashboard" ]
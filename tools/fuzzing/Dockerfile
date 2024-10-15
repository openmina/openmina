FROM debian:bullseye

RUN apt -y update && \
  apt -y upgrade && \
  apt -y install \
    apt-transport-https \
    ca-certificates \
    pkg-config \
    build-essential \
    curl \
    git \
    dnsutils \
    dumb-init \
    gettext \
    gnupg2 \
    unzip \
    bubblewrap \
    jq \
    libgmp10 \
    libgomp1 \
    libssl1.1 \
    libpq-dev \
    libffi-dev \
    libgmp-dev \
    libssl-dev \
    libbz2-dev \
    zlib1g-dev \
    m4 \
    libsodium-dev \
    libjemalloc-dev \
    procps \
    python3 \
    tzdata \
    liblmdb-dev \
    rsync

SHELL ["/bin/bash", "-c"]

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN yes "" | sh <(curl -fsSL https://raw.githubusercontent.com/ocaml/opam/master/shell/install.sh)
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ]; then \
        ARCH="arm64"; \
    elif [ "$ARCH" = "x86_64" ]; then \
        ARCH="amd64"; \
    else \
        echo "Unsupported architecture: $ARCH" && exit 1; \
    fi && \
    rm -rf /usr/local/go && \
    curl -sSL https://go.dev/dl/go1.19.5.linux-$ARCH.tar.gz | tar -C /usr/local -xzf -
RUN export PATH=$PATH:/usr/local/go/bin
RUN curl -sSL https://capnproto.org/capnproto-c++-0.10.2.tar.gz | tar -zxf - \
  && cd capnproto-c++-0.10.2 \
  && ./configure \
  && make -j6 check \
  && make install \
  && cd .. \
  && rm -rf capnproto-c++-0.10.2

RUN git clone https://github.com/openmina/mina.git
WORKDIR /mina
RUN git submodule update --init --recursive && \
    git config --local --add submodule.recurse true && \
    git checkout openmina/fuzzer
RUN opam init --disable-sandboxing && \
    opam switch create .&& \
    eval $(opam config env) && \
    opam switch import -y opam.export && \
    ./scripts/pin-external-packages.sh
RUN source ~/.cargo/env && \
    eval $(opam config env) && \
    export PATH=$PATH:/usr/local/go/bin && \
    export DUNE_PROFILE=devnet && \
    make libp2p_helper && \
    dune build src/app/transaction_fuzzer/transaction_fuzzer.exe --profile=devnet
CMD ["/mina/_build/default/src/app/transaction_fuzzer/transaction_fuzzer.exe", "execute"]


FROM rust:1.77 as builder

WORKDIR /usr/src/openmina-producer-dashboard

COPY ../ .

RUN cd producer-dashboard && SQLX_OFFLINE=true cargo install --path .

FROM minaprotocol/mina-daemon:2.0.0berkeley-rc1-1551e2f-bullseye-berkeley

COPY --from=builder /usr/local/cargo/bin/openmina-producer-dashboard /usr/local/bin/openmina-producer-dashboard

# TODO: replace
ENTRYPOINT [ "/bin/bash" ]
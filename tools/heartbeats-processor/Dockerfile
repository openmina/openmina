# Build stage
FROM rust:1.84-slim-bookworm AS builder

WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN ls -la tools/heartbeats-processor
RUN cargo build --release -p heartbeats-processor

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/heartbeats-processor .
COPY tools/heartbeats-processor/schema.sql .

ENV DATABASE_PATH=/app/data/heartbeats.db

ENTRYPOINT ["./heartbeats-processor"]
CMD ["process-loop"]

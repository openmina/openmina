# Build Stage
FROM rust:bookworm AS builder

WORKDIR /usr/src/app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy files
RUN rm -rf src

# Copy the actual source code
COPY src ./src

RUN ls -la src; cat src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the build stage
COPY --from=builder /usr/src/app/target/release/error-sink-service .

# Create directory for reports (using "reports" instead of "dumps")
RUN mkdir -p /app/reports

# Set environment variables
ENV ERROR_SINK_PORT=8080
ENV ERROR_SINK_DIR=/app/reports
ENV ERROR_SINK_VERIFY_SIGNATURES=true
ENV RUST_LOG=info

# Expose the service port
EXPOSE 8080

# Set the volume for persisting reports
VOLUME /app/reports

# Run the binary
CMD ["./error-sink-service"]

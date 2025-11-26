---
title: Using Docker Images
description: Complete guide to running Mina Rust nodes using Docker images
sidebar_position: 2
---

# Using Docker Images

This guide covers how to run Mina Rust nodes using the pre-built Docker images.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and running
- Basic familiarity with Docker commands
- At least 8GB RAM (16GB recommended for block producers)

## Available Images

<!-- prettier-ignore-start -->

:::warning Deprecated Repository

Docker images from the
[openmina/openmina](https://hub.docker.com/r/openmina/openmina) repository are
deprecated. Please use the
[o1labs/mina-rust](https://hub.docker.com/r/o1labs/mina-rust) images instead for
the latest updates and support.

:::

<!-- prettier-ignore-stop -->

- **Main Node**: `o1labs/mina-rust` - The core Mina Rust node
- **Frontend**: `o1labs/mina-rust-frontend` - Web dashboard and monitoring
  interface

## Image Tags and Versioning

### For Production Use

**Always use version tags for production deployments for stability.** Avoid
using `latest` tags as they may change unexpectedly.

- **Version tags**: `o1labs/mina-rust:v1.4.2` (recommended for stability)
- **Commit-based tags**: `o1labs/mina-rust:2b9e87b2` (available for accessing
  specific features during development, not recommended for general use)

Example:

```bash
# Use a version tag (recommended for stability)
docker pull o1labs/mina-rust:v1.4.2
docker pull o1labs/mina-rust-frontend:v1.4.2

# Commit hashes available for development/testing specific features
docker pull o1labs/mina-rust:2b9e87b2
docker pull o1labs/mina-rust-frontend:2b9e87b2
```

### For Development and Testing

For accessing the latest development features, use the `develop` tag:

<!-- prettier-ignore-start -->

:::warning Unstable Development Version

The `develop` tag points to the latest code from the development branch and may
be unstable. Only use this for development, testing, or accessing the newest
features. For production use, always use version tags.

:::

<!-- prettier-ignore-stop -->

```bash
# Latest development version (unstable)
docker pull o1labs/mina-rust:develop
docker pull o1labs/mina-rust-frontend:develop
```

### Latest Tag

The `latest` tag always corresponds to the latest commit on the main branch,
which represents the current stable release state.

### Finding Available Tags

You can find available tags at:

- [o1labs/mina-rust on Docker Hub](https://hub.docker.com/r/o1labs/mina-rust/tags)
- [o1labs/mina-rust-frontend on Docker Hub](https://hub.docker.com/r/o1labs/mina-rust-frontend/tags)

## Architecture Support

All Docker images are built natively for multiple architectures to ensure
optimal performance:

- **`linux/amd64`** (x86_64) - For Intel/AMD processors
- **`linux/arm64`** (ARM64) - For ARM processors (Apple Silicon, AWS Graviton,
  etc.)

### Automatic Architecture Selection

Docker automatically pulls the correct architecture for your system:

```bash
# This automatically pulls the right architecture
docker pull o1labs/mina-rust:latest

# On Intel/AMD systems: gets linux/amd64
# On Apple Silicon: gets linux/arm64
# On ARM servers: gets linux/arm64
```

### Performance Benefits

- **Native builds**: Each architecture is compiled natively for optimal
  performance
- **No emulation overhead**: ARM users get native performance instead of x86
  emulation
- **Faster startup**: Native images start faster than emulated ones

## Quick Start Examples

### Basic Node (Testing/Development)

```bash
# Pull and run a basic node for testing
docker pull o1labs/mina-rust:latest
docker run -p 8302:8302 o1labs/mina-rust:latest
```

### Node with Frontend Dashboard

```bash
# Run node and web dashboard together
docker run -d --name mina-rust-node -p 8302:8302 o1labs/mina-rust:latest \
  node --network devnet
# Frontend
docker run -d --name mina-frontend -p 8070:8070 o1labs/mina-rust-frontend:latest
```

### Using Docker Compose (Recommended)

Docker Compose provides the easiest way to run both the Mina node and frontend
dashboard together.

#### Quick Start with Release Version

For each release, you can directly download the docker-compose.yml file:

```bash
# Create a directory for your Mina node
mkdir mina-rust-node && cd mina-rust-node

# Download the docker-compose.yml file (choose one method)
# Using wget:
wget https://raw.githubusercontent.com/o1-labs/mina-rust/v0.18.0/docker-compose.yml

# Or using curl:
curl -O https://raw.githubusercontent.com/o1-labs/mina-rust/v0.18.0/docker-compose.yml

# Create an empty .env file to avoid warnings (optional - has defaults)
touch .env

# Or create .env with custom settings (optional)
cat > .env << EOF
MINA_RUST_TAG=latest
MINA_FRONTEND_TAG=latest
MINA_LIBP2P_PORT=8302
EOF

# Start the node
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

#### Using Latest Development Version

To use the latest development version with all configuration files:

```bash
# Clone the repository to get the docker compose configuration
git clone https://github.com/o1-labs/mina-rust.git
cd mina-rust

# Start both node and frontend using docker compose
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

#### Configuration Options

The docker compose setup supports several environment variables:

```bash
# Use specific versions (recommended for production)
MINA_RUST_TAG=v1.4.2 MINA_FRONTEND_TAG=v1.4.2 docker compose up -d

# Use development version (latest features, unstable)
MINA_RUST_TAG=develop MINA_FRONTEND_TAG=develop docker compose up -d

# Configure custom libp2p settings
MINA_LIBP2P_PORT=9302 MINA_LIBP2P_EXTERNAL_IP=203.0.113.1 docker compose up -d
```

#### What's Included

- **mina-rust-node**: The core Mina Rust node running on devnet
- **frontend**: Web dashboard accessible at http://localhost:8070
- **Persistent storage**: Node data stored in `./mina-workdir` directory
- **Automatic networking**: Services can communicate with each other

#### Benefits

- **Easy setup**: Single command to start both services
- **Configuration management**: Environment variables for customization
- **Service orchestration**: Automatic startup order and networking
- **Data persistence**: Blockchain data survives container restarts

### Advanced Configuration

```bash
# Run with custom verbosity
docker run -d --name mina-rust-node \
  -p 8302:8302 \
  o1labs/mina-rust:latest \
  node --network devnet --verbosity debug

# Run with external IP and custom libp2p port
# Get your external IP first
EXTERNAL_IP=$(curl -s https://ifconfig.me/ip)
docker run -d --name mina-rust-node \
  -p 9302:9302 \
  -e MINA_LIBP2P_EXTERNAL_IP=$EXTERNAL_IP \
  o1labs/mina-rust:latest \
  node --network devnet --libp2p-port 9302
```

### Verifying Docker Image Version

Always verify the version of your Docker image to ensure you're running the
correct build:

```bash
# Check build information
docker run --rm o1labs/mina-rust:latest build-info

# For a specific version
docker run --rm o1labs/mina-rust:v0.17.0 build-info
```

This displays version details including commit hash, build time, and Rust
compiler version. See [Node Management](node-management) for more details.

## Next Steps

For specific node configurations and detailed setup guides:

- [Block Producer Setup](block-producer) - Complete guide for running a block
  producer
- [Archive Node Setup](archive-node) - Complete guide for running an archive
  node
- [Docker Installation](../appendix/docker-installation) - Installing Docker on
  your system

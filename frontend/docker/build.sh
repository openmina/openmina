#!/usr/bin/env bash

set -e

# Navigate to the project root
cd "$(dirname "$0")/../.."

# Default values
DOCKER_ORG=${DOCKER_ORG:-o1labs}
NODE_VERSION=${NODE_VERSION:-$(cat .nvmrc)}
GIT_COMMIT=${GIT_COMMIT:-$(git rev-parse --short=8 HEAD)}
IMAGE_TAG=${1:-$DOCKER_ORG/mina-rust-frontend:$GIT_COMMIT}

echo "Building frontend Docker image..."
echo "Node Version: $NODE_VERSION"
echo "Image Tag: $IMAGE_TAG"

# Ensure circuits are present
echo "Ensuring circuit blobs are present..."
make download-circuits

# Generate .env.docker
echo "Generating .env.docker..."
./frontend/docker/generate-docker-env.sh

# Detect platform
ARCH=$(uname -m)
case $ARCH in
    x86_64) PLATFORM="linux/amd64" ;;
    aarch64|arm64) PLATFORM="linux/arm64" ;;
    *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

echo "Building for platform: $PLATFORM"

# Build the image using buildx and load it to the local Docker daemon
docker buildx build \
    --build-arg NODE_VERSION="$NODE_VERSION" \
    --platform "$PLATFORM" \
    --tag "$IMAGE_TAG" \
    --file ./frontend/Dockerfile \
    --load \
    ./

echo "Done. Image built as '$IMAGE_TAG'."
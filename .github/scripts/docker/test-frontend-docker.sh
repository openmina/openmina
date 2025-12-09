#!/bin/bash
# Test frontend Docker image with specific environment configuration
# Usage: ./test-frontend-docker.sh <image> <environment> [port]
#
# Parameters:
#   image       - Docker image name/tag to test
#   environment - Frontend environment configuration (local, webnode, production, development, producer, fuzzing)
#   port        - Optional port to use (default: 8080)
#
# Examples:
#   ./test-frontend-docker.sh o1labs/mina-rust-frontend:latest production
#   ./test-frontend-docker.sh test-frontend:local local 9090

set -euo pipefail

# Check arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <image> <environment> [port]"
    echo ""
    echo "Supported environments: local, webnode, production, fuzzing, staging"
    echo ""
    echo "Examples:"
    echo "  $0 o1labs/mina-rust-frontend:latest production"
    echo "  $0 test-frontend:local local 9090"
    exit 1
fi

IMAGE="$1"
ENVIRONMENT="$2"
PORT="${3:-8080}"
CONTAINER_NAME="test-frontend-${ENVIRONMENT}-$$"

# Supported environments
SUPPORTED_ENVS="local webnode production fuzzing staging"
if [[ ! " $SUPPORTED_ENVS " =~ \ $ENVIRONMENT\  ]]; then
    echo "❌ Unsupported environment: $ENVIRONMENT"
    echo "Supported environments: $SUPPORTED_ENVS"
    exit 1
fi

echo "🧪 Testing frontend image with environment: $ENVIRONMENT"
echo "📦 Image: $IMAGE"
echo "🔌 Port: $PORT"
echo "📝 Container: $CONTAINER_NAME"
echo ""

# Cleanup function
cleanup() {
    echo "🧹 Cleaning up container: $CONTAINER_NAME"
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
}

# Set up cleanup trap
trap cleanup EXIT

# Check if port is available
if ss -tuln | grep -q ":$PORT "; then
    echo "❌ Port $PORT is already in use"
    exit 1
fi

# Run the container with the specific environment configuration
echo "🚀 Starting container..."
docker run --rm -d \
    --name "$CONTAINER_NAME" \
    -p "$PORT:80" \
    -e MINA_FRONTEND_ENVIRONMENT="$ENVIRONMENT" \
    "$IMAGE"

# Wait a moment for container to start
echo "⏳ Waiting for container to initialize..."
sleep 10

# Check if container is running
if ! docker ps | grep -q "$CONTAINER_NAME"; then
    echo "❌ Container failed to start with environment: $ENVIRONMENT"
    echo "📋 Container logs:"
    docker logs "$CONTAINER_NAME" || echo "No logs available"
    exit 1
fi

echo "✅ Container started successfully with environment: $ENVIRONMENT"

# Test HTTP endpoint with retries (30 attempts with 3 second intervals = ~90s total)
RETRY_COUNT=0
MAX_RETRIES=30
SUCCESS=false

echo "🔍 Testing HTTP endpoint with retries (max $MAX_RETRIES attempts)..."

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -f -s -m 5 "http://localhost:$PORT/" > /dev/null 2>&1; then
        echo "✅ HTTP endpoint is responding for environment: $ENVIRONMENT (attempt $((RETRY_COUNT + 1)))"
        SUCCESS=true
        break
    else
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
            echo "❌ HTTP endpoint not ready after $MAX_RETRIES attempts"
        else
            echo "⏳ HTTP endpoint not ready, attempt $RETRY_COUNT/$MAX_RETRIES for environment: $ENVIRONMENT"
            sleep 3
        fi
    fi
done

if [ "$SUCCESS" = false ]; then
    echo "❌ HTTP endpoint failed after $MAX_RETRIES attempts for environment: $ENVIRONMENT"
    echo "📋 Container logs:"
    docker logs "$CONTAINER_NAME"
    exit 1
fi

echo "🎉 Test completed successfully for environment: $ENVIRONMENT"
echo ""
echo "🌐 Frontend is available at: http://localhost:$PORT/"
echo "🔍 To view logs: docker logs $CONTAINER_NAME"
echo "🛑 Container will be automatically stopped when script exits"
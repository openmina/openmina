#!/bin/bash

set -e

# Function to build frontend based on environment
build_frontend() {
    local environment="$1"
    echo "Building frontend for environment: $environment"

    cd /app/frontend

    # Source NVM to make Node.js and npm available
    export NVM_DIR="$HOME/.nvm"
    # shellcheck disable=SC1091
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

    # Map environment to Makefile targets and Angular configurations
    case "$environment" in
        "local")
            # Uses Angular local configuration and local.js runtime
            make build-local
            ;;
        "fuzzing")
            # Uses Angular fuzzing configuration and fuzzing.js runtime
            make build-fuzzing
            ;;
        "production")
            # Uses Angular production configuration and production.js runtime
            make build-production
            ;;
        "webnode")
            # Uses Angular webnode-local configuration and webnode.js runtime
            make build-webnode
            ;;
        "staging")
            # Uses Angular production configuration with staging.js runtime
            make build-staging
            ;;
        *)
            echo "Error: Unknown environment '$environment'"
            echo "Available environments: local, fuzzing, production, webnode, staging"
            exit 1
            ;;
    esac

    # Copy built files to Apache document root
    echo "Copying built files to Apache document root..."
    cp -r dist/frontend/browser/* /usr/local/apache2/htdocs/

    echo "Frontend build complete for environment: $environment"
    cd /app
}

# Validate that MINA_FRONTEND_ENVIRONMENT is set
if [ -z "$MINA_FRONTEND_ENVIRONMENT" ]; then
    echo "Error: MINA_FRONTEND_ENVIRONMENT environment variable is required."
    echo "Available environments: local, fuzzing, production, webnode, staging"
    echo "Example: docker run -e MINA_FRONTEND_ENVIRONMENT=webnode mina-frontend"
    exit 1
fi

echo "Using environment: $MINA_FRONTEND_ENVIRONMENT"

# Build the frontend for the specified environment
build_frontend "$MINA_FRONTEND_ENVIRONMENT"

# Environment file is now copied by Makefile build targets
echo "Environment configuration set during build process"

echo "Starting Apache..."
exec "$@"

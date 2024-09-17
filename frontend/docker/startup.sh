#!/bin/bash

if [ -n "$OPENMINA_FRONTEND_ENVIRONMENT" ]; then
  echo "Using environment: $OPENMINA_FRONTEND_ENVIRONMENT"
  cp -f /usr/local/apache2/htdocs/assets/environments/"$OPENMINA_FRONTEND_ENVIRONMENT".js \
        /usr/local/apache2/htdocs/assets/environments/env.js
else
  echo "No environment specified. Using default."
fi

echo "Starting Apache..."
exec "$@"

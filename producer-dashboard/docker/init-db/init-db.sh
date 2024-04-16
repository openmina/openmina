#!/bin/bash
set -e

# Configuration
HOST=${PG_HOST}
PORT=${PG_PORT}
DB=${PG_DB}
USER=${PG_USER}
PASSWORD=${POSTGRES_PASSWORD}

# Wait for PostgreSQL to become available
echo "Waiting for postgres..."
while ! pg_isready -h $HOST -p $PORT -U $USER; do
  sleep 2;
done

# Check if the database exists, and create it if it doesn't
if ! PGPASSWORD=$PASSWORD psql -h $HOST -U $USER -lqt | cut -d \| -f 1 | grep -qw $DB; then
  echo "Database $DB does not exist. Creating..."
  PGPASSWORD="$PASSWORD" createdb -h "$HOST" -U "$USER" "$DB"
  # Load the schema into the database
  echo "Loading schema into $DB..."
  cd /init-db
  # PGPASSWORD=$PASSWORD psql -h $HOST -U $USER -d $DB < create_schema.sql
  # TODO
  PGPASSWORD=$PASSWORD psql -h $HOST -U $USER -d $DB < dump.sql
  exit 1
else
  echo "Database $DB already exists."
  exit 1
fi

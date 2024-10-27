#!/bin/bash

# Bring up the test database and run the tests

cd "$(dirname "$0")/.."

export COMPOSE_FILE=docker-compose-test.yaml

. scripts/_await_service.sh

docker compose up -d
await_service test_db 'ready to accept connections'

cargo test || {
  echo 'Leaving test database running to aid debugging' >&1
  exit "$?"
}

echo 'Destroying test db...'
docker compose down --volumes

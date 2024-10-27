#!/bin/bash

# Bootstrap a basic dev environment by bringing up the primary docker compose
# and waiting for the database to be ready. You can then run the app when ready.

cd "$(dirname "$0")/.."

. scripts/_await_service.sh

docker compose up -d
await_service rusteam_db 'ready to accept connections'

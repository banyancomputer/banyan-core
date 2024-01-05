#!/usr/bin/env bash

set -o errexit

# Relative paths don't work reliably with sqlx
export DATABASE_URL="sqlite://$(pwd)/data/server.db"

rm -f data/server.db* &>/dev/null

cargo sqlx database setup
sqlite3 data/server.db 'PRAGMA journal_mode=WAL'
cargo sqlx prepare -- --all-targets --all-features --tests

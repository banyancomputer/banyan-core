#!/bin/bash

set -o errexit
set -o nounset

EMAIL_ADDRESS="${1:-}"

if [ -z "${EMAIL_ADDRESS}" ]; then
  echo "Need to provide email address as first argument"
  exit 1
fi

echo "INSERT INTO allowed_emails (email) VALUES ('${EMAIL_ADDRESS}');" | sqlite3 ../data/server.db
echo 'Done'

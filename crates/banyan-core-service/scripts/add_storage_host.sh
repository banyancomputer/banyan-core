#!/bin/bash

export STORAGE_HOST_PUBKEY=$(cat ../banyan-staging-service/data/platform-auth.public)
export STORAGE_HOST_FINGERPRINT=$(cat ../banyan-staging-service/data/platform-auth.fingerprint)

cat << EOSQL | sqlite3 ./data/server.db
DELETE FROM storage_hosts;
INSERT INTO storage_hosts (name, url, used_storage, available_storage, fingerprint, pem)
  VALUES ("banyan-staging", "http://127.0.0.1:3002", 0, 549755813888000, "${STORAGE_HOST_FINGERPRINT}", "${STORAGE_HOST_PUBKEY}")
  RETURNING id;
EOSQL

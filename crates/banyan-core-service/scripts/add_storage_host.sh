#!/bin/bash

export STORAGE_HOST_PUBKEY=$(cat ../banyan-staging-service/data/platform.public)
export STORAGE_HOST_FINGERPRINT=$(cat ../banyan-staging-service/data/platform.fingerprint)

cat << EOSQL | sqlite ./data/server.db
DELETE FROM storage_hosts;
INSERT INTO storage_hosts (name, url, available_storage, fingerprint, pem)
  VALUES ("banyan-staging", "http://127.0.0.1:3002", 549755813888000, "${STORAGE_HOST_FINGERPRINT}", "${STORAGE_HOST_PUBKEY}")
  RETURNING id;
EOSQL

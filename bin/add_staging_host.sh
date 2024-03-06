#!/usr/bin/env bash

export STAGING_HOST_PUBKEY="$(cat crates/banyan-staging-service/data/service-key.public)"
export STAGING_HOST_FINGERPRINT="$(cat crates/banyan-staging-service/data/service-key.fingerprint)"
export STAGING_HOST_NAME="banyan-staging"
export STAGING_HOST_REGION="North America"
export STAGING_HOST_URL="http://127.0.0.1:3002/"
export STAGING_HOST_BYTE_LIMIT="549755813888000"

cat <<ESQL | sqlite3 ./crates/banyan-core-service/data/server.db || fail 4 "creating staging server storage host record"
INSERT INTO storage_hosts
  (name, url, used_storage, reserved_storage, available_storage, region, fingerprint, pem, staging)
  VALUES ('${STAGING_HOST_NAME}', '${STAGING_HOST_URL}', 0, 0, ${STAGING_HOST_BYTE_LIMIT}, '${STAGING_HOST_REGION}', '${STAGING_HOST_FINGERPRINT}', '${STAGING_HOST_PUBKEY}', TRUE);
ESQL

#!/usr/bin/env bash

export STORAGE_HOST_PUBKEY="$(cat crates/banyan-storage-provider-service/data/service-key.public)"
export STORAGE_HOST_FINGERPRINT="$(cat crates/banyan-storage-provider-service/data/service-key.fingerprint)"
export STORAGE_HOST_NAME="banyan-storage-provider"
export STORAGE_HOST_URL="http://127.0.0.1:3003/"
export STORAGE_HOST_BYTE_LIMIT="549755813888000"

cat <<ESQL | sqlite3 ./crates/banyan-core-service/data/server.db || fail 7 "creating storage server storage host record"
INSERT INTO storage_hosts
  (name, url, used_storage, available_storage, fingerprint, pem)
  VALUES ('${STORAGE_HOST_NAME}', '${STORAGE_HOST_URL}', 0, ${STORAGE_HOST_BYTE_LIMIT}, '${STORAGE_HOST_FINGERPRINT}', '${STORAGE_HOST_PUBKEY}');
ESQL
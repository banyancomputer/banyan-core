#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

cd $(pwd)

function fail() {
	local status_code=${1:-1}
	local err_msg="${2:-an unknown error occurred}"

	echo "ERROR: ${err_msg}"
	exit $status_code
}

rm -rf \
	crates/banyan-core-service/data/serv* \
	crates/banyan-core-service/data/uploads/*

rm -rf crates/banyan-staging-service/data/serv* \
	crates/banyan-staging-service/data/platform* \
	crates/banyan-staging-service/data/uploads/*

rm -rf crates/banyan-storage-provider-service/data/serv* \
	crates/banyan-storage-provider-service/data/platform* \
	crates/banyan-storage-provider-service/data/uploads/*

# Run core
(
	cd crates/banyan-core-service
	cargo build
	timeout 3s cargo run || true
)

# Copy core public key to staging and storage provider
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-staging-service/data/platform-key.public
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-storage-provider-service/data/platform-key.public

# Run staging
(
	cd crates/banyan-staging-service
	cargo build
	timeout 3s cargo run || true
)

[ -f "crates/banyan-staging-service/data/service-key.public" ] || fail 1 "staging missing public service key"
[ -f "crates/banyan-staging-service/data/service-key.fingerprint" ] || fail 2 "staging missing service fingerprint"

export STAGING_HOST_PUBKEY="$(cat crates/banyan-staging-service/data/service-key.public)"
export STAGING_HOST_FINGERPRINT="$(cat crates/banyan-staging-service/data/service-key.fingerprint)"
export STAGING_HOST_NAME="banyan-staging"
export STAGING_HOST_URL="http://127.0.0.1:3002/"
export STAGING_HOST_BYTE_LIMIT="549755813888000"

cat <<ESQL | sqlite3 ./crates/banyan-core-service/data/server.db || fail 3 "creating staging server storage host record"
INSERT INTO storage_hosts
  (name, url, used_storage, available_storage, fingerprint, pem)
  VALUES ('${STAGING_HOST_NAME}', '${STAGING_HOST_URL}', 0, ${STAGING_HOST_BYTE_LIMIT}, '${STAGING_HOST_FINGERPRINT}', '${STAGING_HOST_PUBKEY}');
ESQL

# Run storage provider
(
	cd crates/banyan-storage-provider-service
	cargo build
	timeout 3s cargo run || true
)

[ -f "crates/banyan-storage-provider-service/data/service-key.public" ] || fail 4, "storage provider missing public service key"
[ -f "crates/banyan-storage-provider-service/data/service-key.fingerprint" ] || fail 5, "storage provider missing service fingerprint"

export STORAGE_HOST_PUBKEY="$(cat crates/banyan-storage-provider-service/data/service-key.public)"
export STORAGE_HOST_FINGERPRINT="$(cat crates/banyan-storage-provider-service/data/service-key.fingerprint)"
export STORAGE_HOST_NAME="banyan-storage-provider"
export STORAGE_HOST_URL="http://127.0.0.1:3003/"
export STORAGE_HOST_BYTE_LIMIT="549755813888000"

cat <<ESQL | sqlite3 ./crates/banyan-core-service/data/server.db || fail 6 "creating storage server storage host record"
INSERT INTO storage_hosts
  (name, url, used_storage, available_storage, fingerprint, pem)
  VALUES ('${STORAGE_HOST_NAME}', '${STORAGE_HOST_URL}', 0, ${STORAGE_HOST_BYTE_LIMIT}, '${STORAGE_HOST_FINGERPRINT}', '${STORAGE_HOST_PUBKEY}');
ESQL

echo 'environment reset complete'

#!/usr/bin/env bash
#
# NB: when changes are made to this file, be sure to update the introductory
# guide in the project readme.

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

# Check this script has been invoked from the root of the repository.
if [ $(basename "$PWD") != "banyan-core" ]; then
	fail 9 "this script should be invoked from the root of the banyan-core repository"
fi

# Remove any state from previous runs before we do anything else.
make clean

# Make sure object storage is up and running.
make minio

# Generate the core service public key, copy it to the staging and storage provider services.
make generate-core-service-key

# Confirm the command correctly created what we need from it
[ -f "crates/banyan-core-service/data/service-key.public" ] || fail 1 "core didn't generate public key"

# Copy the public key to the other services for verification
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-staging-service/data/platform-key.public
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-storage-provider-service/data/platform-key.public

# Generate the staging service's public key and its fingerprint. Then, add the staging host to the sqlite database.
source crates/banyan-staging-service/.env
make generate-staging-service-key

# Confirm the command correctly created what we need from it
[ -f "crates/banyan-staging-service/data/service-key.public" ] || fail 2 "staging missing public service key"
[ -f "crates/banyan-staging-service/data/service-key.fingerprint" ] || fail 3 "staging missing service fingerprint"

./bin/add_staging_host.sh

# Generate the storage provider service's public key and its fingerprint. Then, add the storage host to the sqlite database.
source crates/banyan-storage-provider-service/.env
make generate-storage-provider-service-key

# Confirm the command correctly created what we need from it
[ -f "crates/banyan-storage-provider-service/data/service-key.public" ] || fail 5 "storage provider missing public service key"
[ -f "crates/banyan-storage-provider-service/data/service-key.fingerprint" ] || fail 6 "storage provider missing service fingerprint"

./bin/add_storage_host.sh

echo 'environment reset complete'

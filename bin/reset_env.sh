#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

cd $(pwd)
pwd

EMAIL_TO_ALLOW="${1:-}"
if [ -z "${EMAIL_TO_ALLOW}" ]; then
  echo "Need to provide an email to allow as the first argument"
  exit 1
fi

rm -rf crates/banyan-core-service/data/s* \
  crates/banyan-core-service/data/uploads/* \
  crates/banyan-staging-service/data/pl* \
  crates/banyan-staging-service/data/uploads/*

(cd crates/banyan-core-service; cargo build; timeout 3s cargo run || true)
cp -f crates/banyan-core-service/data/signing-key.public crates/banyan-staging-service/data/platform-verifier.public

(cd crates/banyan-staging-service; cargo build; timeout 45s cargo run -- --generate-auth)

#(cd crates/banyan-core-server/frontend; yarn install; source .env.dev; timeout 5s yarn run dev)

[ ! -f "crates/banyan-staging-service/data/platform-auth.public" ] && exit 8
[ ! -f "crates/banyan-staging-service/data/platform-auth.fingerprint" ] && exit 9

export STORAGE_HOST_PUBKEY="$(cat crates/banyan-staging-service/data/platform-auth.public)"
export STORAGE_HOST_FINGERPRINT="$(cat crates/banyan-staging-service/data/platform-auth.fingerprint)"
export STORAGE_HOST_NAME="banyan-staging"
export STORAGE_HOST_URL="http://127.0.0.1:3002"
export STORAGE_HOST_BYTE_LIMIT="549755813888000"

cat << ESQL | sqlite3 ./crates/banyan-core-service/data/server.db
INSERT INTO storage_hosts
  (name, url, used_storage, available_storage, fingerprint, pem)
  VALUES ('${STORAGE_HOST_NAME}', '${STORAGE_HOST_URL}', 0, ${STORAGE_HOST_BYTE_LIMIT}, '${STORAGE_HOST_FINGERPRINT}', '${STORAGE_HOST_PUBKEY}');

INSERT INTO allowed_emails (email) VALUES ('${EMAIL_TO_ALLOW}');
ESQL

NEW_NEXTAUTH_SECRET="$(openssl rand -base64 9 | tr -dc 'a-zA-Z0-9' | cut -c1-12)"
sed -i "s/^export NEXTAUTH_SECRET=.*/export NEXTAUTH_SECRET=${NEW_NEXTAUTH_SECRET}/" crates/banyan-core-service/frontend/.env.dev

echo 'environment reset complete'

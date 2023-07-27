#!/bin/bash

set -o errexit
set -o pipefail

BASE_HOST="http://127.0.0.1:3000"

TMP_CERT_DIR="/tmp/ec_certs_gen"
PRIVATE_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/private.ec.key"
PUBLIC_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/public.ec.pem"

mkdir -p $(dirname "${PRIVATE_EC_CLIENT_KEY_PATH}")
openssl ecparam -name secp384r1 -genkey -noout -out "${PRIVATE_EC_CLIENT_KEY_PATH}" 2>/dev/null
openssl ec -in "${PRIVATE_EC_CLIENT_KEY_PATH}" -pubout -out "${PUBLIC_EC_CLIENT_KEY_PATH}" 2>/dev/null

# Register account, get generic authentication token
REGISTER_RESPONSE="$(curl -s -H "Content-Type: application/json" ${BASE_HOST}/api/v1/auth/fake_register)"

ACCOUNT_ID="$(echo "${REGISTER_RESPONSE}" | jq -r .id)"
ACCOUNT_TOKEN="$(echo "${REGISTER_RESPONSE}" | jq -r .token)"

# Register a device key using regular authentication token
KEY_REG_DATA="{\"public_key\":\"$(cat ${PUBLIC_EC_CLIENT_KEY_PATH} | sed ':a;N;$!ba;s/\n/\\n/g')\"}"
REGISTERED_FINGERPRINT="$(curl -s -H "Content-Type: application/json" -H "Authorization: Bearer ${ACCOUNT_TOKEN}" -X POST -d "${KEY_REG_DATA}" ${BASE_HOST}/api/v1/auth/register_device_key | jq -r .fingerprint)"

# Some dirty stuff to generate a JWT using only bash tooling...
HEADER="$(echo "{\"typ\":\"JWT\",\"alg\":\"ES384\",\"kid\":\"${REGISTERED_FINGERPRINT}\"}" | base64 -w 0 | tr '+/' '-_' | tr -d '=')"
# Should be 8 random bytes hex encoded
NONCE="$(openssl rand -hex 8)"
EXPIRATION_UNIX_TIME="$(($(date +%s) + 600))"
TOKEN_BODY="$(echo "{\"nnc\":\"${NONCE}\",\"exp\":${EXPIRATION_UNIX_TIME},\"nbf\":$(date +%s),\"aud\":\"banyan-platform\",\"sub\":\"${ACCOUNT_ID}\"}" | base64 -w 0 | tr '+/' '-_' | tr -d '=')"
SIGNED_BODY="$(echo "${HEADER}.${TOKEN_BODY}")"

# Note: This signature generation isn't quite working yet, example generation
# in rust is available in src/api/auth/handlers.rs (see fake_register, though
# the format is of the header and body of the token are correct here in this
# script).
SIGNATURE=$(echo -e ${SIGNED_BODY} | openssl dgst -sha384 -binary -sign ${TMP_CERT_DIR}/private.ec.key | base64 -w 0 | tr '+/' '-_' | tr -d '=')

AUTH_TOKEN="${SIGNED_BODY}.${SIGNATURE}"

# Create a bucket using device key authentication (associated to account) and an initial public encryption key
BUCKET_CREATION_PAYLOAD="{\"friendly_name\":\"Sweet Interactive Bucket\","type":\"interactive\",\"initial_public_key\":\"<replace with ec signing key>\"}"
BUCKET_CREATION_RESPONSE="$(curl -s -H "Authorization: Bearer ${AUTH_TOKEN}" -X POST -d "${BUCKET_CREATION_PAYLOAD}" ${BASE_HOST}/api/v1/buckets)"
BUCKET_ID="$(echo ${BUCKET_CREATION_RESPONSE} | jq -r .id)"
echo "Created bucket ID: ${BUCKET_ID}"

# Retrieve information about the created bucket using device key authentication
BUCKET_CHECK_RESPONSE="$(curl -s -H "Authorization: Bearer ${AUTH_TOKEN}" ${BASE_HOST}/api/v1/buckets/${BUCKET_ID})"
echo ${BUCKET_CHECK_RESPONSE}

# Publish metadata for the bucket using the device key
cat <<EOF | curl -s -H "Authorization: Bearer ${AUTH_TOKEN}" -H "Content-Type: application/vnd.ipld.car; version=2" --data-binary "@-" ${BASE_HOST}/api/v1/buckets/${BUCKET_ID}/publish
# This should be a CARv2 file, but alas its just a placeholder x

This data file was generated at $(date +%s.%N) or $(date).

Your fortune (if available):

$(fortune 2>/dev/null || echo "No fortune for you...")
EOF

# Retrieve metadata for the bucket using device key
curl -s -H "Authorization: Bearer ${AUTH_TOKEN}" -o ${TMP_CERT_DIR}/metadata.car ${BASE_HOST}/api/v1/buckets/${BUCKET_ID}/metadata

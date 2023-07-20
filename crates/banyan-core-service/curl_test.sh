#!/bin/bash

set -o errexit
set -o pipefail

BASE_HOST="http://127.0.0.1:3000"
BUCKET_ID="$(uuidgen)"

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
HEADER="$(echo "{\"typ\":\"JWT\",\"alg\":\"ES384\",\"kid\":\"${REGISTERED_FINGERPRINT}\"}" | base64 -w 0)"
NONCE="$(openssl rand -hex 8)" # Should be 8 random bytes hex encoded
EXPIRATION_UNIX_TIME="$(($(date +%s) + 600))"
TOKEN_BODY="$(echo "{\"nnc\":\"${NONCE}\",\"exp\":${EXPIRATION_UNIX_TIME},\"nbf\":$(date +%s),\"aud\":\"banyan-platform\",\"sub\":\"${ACCOUNT_ID}\"}" | base64 -w 0)"
SIGNED_BODY="$(echo "${HEADER}.${TOKEN_BODY}")"
SIGNATURE=$(echo -e ${SIGNED_BODY} | openssl dgst -sha384 -binary -sign ${TMP_CERT_DIR}/private.ec.key | base64 -w 0)

AUTH_TOKEN="${TOKEN_BODY}.${SIGNATURE}"


# Create a bucket using device key authentication (associated to account) and an initial public encryption key
# Retrieve information about the created bucket using device key authentication
#
# Attempt to retrieve metadata for the bucket using device key authentication, it should 404
#
# Publish metadata for the bucket using the device key
# Retrieve metadata for the bucket using device key, it should succeed and match the bytes that were uploaded
#
# Publish new metadata for the bucket using the device key
# Retrieve metadata for the bucket using device key, it should succeed and match the most recent version

cat <<EOF | curl -s -H "Authorization: Bearer ${ACCOUNT_TOKEN}" -H "Content-Type: application/vnd.ipld.car; version=2" --data-binary "@-" ${BASE_HOST}/api/v1/buckets/${BUCKET_ID}/publish
# This should be a CARv2 file, but alas its just a placeholder x

This data file was generated at $(date +%s.%N) or $(date).

Your fortune (if available):

$(fortune 2>/dev/null || echo "No fortune for you...")
EOF
#echo


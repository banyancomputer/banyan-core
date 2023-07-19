#!/bin/bash

BASE_HOST="http://127.0.0.1:3000"
API_TOKEN="$(curl -s ${BASE_HOST}/api/v1/auth/fake_token)"
BUCKET_ID="$(uuidgen)"

PRIVATE_EC_CLIENT_KEY_PATH="/tmp/ec_certs_gen/private.ec.key"
PUBLIC_EC_CLIENT_KEY_PATH="/tmp/ec_certs_gen/public.ec.pem"

if [ ! -f "${PRIVATE_EC_CLIENT_KEY_PATH}" ]; then
  mkdir -p $(dirname "${PRIVATE_EC_CLIENT_KEY_PATH}")
  openssl ecparam -name secp384r1 -genkey -noout -out "${PRIVATE_EC_CLIENT_KEY_PATH}"
  openssl ec -in "${PRIVATE_EC_CLIENT_KEY_PATH}" -pubout -out "${PUBLIC_EC_CLIENT_KEY_PATH}"
fi

RAW_FINGERPRINT="$(openssl ec -pubin -in "${PUBLIC_EC_CLIENT_KEY_PATH}" -outform der 2>/dev/null | dd ibs=26 skip=1 2>/dev/null | openssl dgst -sha1 | cut -d ' ' -f 2)"
FINGERPRINT="$(echo "${RAW_FINGERPRINT}" | sed 's/../&:/g; s/:$//')"

cat <<EOF | curl -s -H "Authorization: Bearer ${API_TOKEN}" -H "Content-Type: application/vnd.ipld.car; version=2" --data-binary "@-" ${BASE_HOST}/api/v1/buckets/${BUCKET_ID}/publish
# This should be a CARv2 file, but alas its just a placeholder x  

This data file was generated at $(date +%s.%N) or $(date).

Your fortune (if available):

$(fortune 2>/dev/null || echo "No fortune for you...")
EOF
echo


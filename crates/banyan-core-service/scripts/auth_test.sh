#!/bin/bash

set -o errexit
set -o pipefail

FRONTEND_HOST="http://localhost:3000"
BASE_HOST="http://127.0.0.1:3001"

# These should already bew set up from the register_test.sh script
TMP_CERT_DIR="/tmp/ec_certs_gen"
PRIVATE_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/private.ec.key"
PUBLIC_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/public.ec.pem"
openssl ec -in "${PRIVATE_EC_CLIENT_KEY_PATH}" -pubout -out "${PUBLIC_EC_CLIENT_KEY_PATH}" 2>/dev/null

# TODO: This eventually should be generated from the public key -- for now you'll need to set this in the env 
REGISTERED_FINGERPRINT=$(node scripts/js/fingerprint.js ${PUBLIC_EC_CLIENT_KEY_PATH})
echo "Using Registered key with id: $REGISTERED_FINGERPRINT"
echo "Using subject: $ACCOUNT_ID"
if [ -z "$REGISTERED_FINGERPRINT" ]; then
  echo "You need to register a key first!"
  exit 1
fi
if [ -z "$ACCOUNT_ID" ]; then
  echo "You need to set the ACCOUNT_ID env variable!"
  exit 1
fi

NONCE="$(openssl rand -hex 8)"
EXPIRATION_UNIX_TIME="$(($(date +%s) + 600))"

AUTH_TOKEN=$(node scripts/js/jwt.js ${PRIVATE_EC_CLIENT_KEY_PATH} ${REGISTERED_FINGERPRINT} ES384 '{"nnc":"'"${NONCE}"'","exp":'"${EXPIRATION_UNIX_TIME}"',"nbf":'"$(date +%s)"',"aud":"banyan-platform","sub":"'"${ACCOUNT_ID}"'"}')

# Create a bucket using device key authentication (associated to account) and an initial public encryption key
BUCKET_CREATION_PAYLOAD="{\"name\":\"Sweet Interactive Bucket\","type":\"interactive\",\"initial_public_key\":\"${PUBLIC_EC_CLIENT_KEY_PEM}\"}"
CHECK="$(curl -s -o /dev/null -w "%{http_code}" -H "Content-Type: application/json" -H "Authorization: Bearer ${AUTH_TOKEN}" -X POST -d "${BUCKET_CREATION_PAYLOAD}" ${BASE_HOST}/api/v1/buckets)"
if [ "$CHECK" == "401" ]; then
  echo "Failed Auth"
  exit 1
fi
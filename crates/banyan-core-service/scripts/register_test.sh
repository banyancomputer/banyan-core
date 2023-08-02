#!/bin/bash

set -o errexit
set -o pipefail

# You should habve an authenticated session in firefox before running this script
# Important that this specifies localhost in order for cookies to work
FRONTEND_HOST="http://localhost:3000"

TMP_CERT_DIR="/tmp/ec_certs_gen"
PRIVATE_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/private.ec.key"
PUBLIC_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/public.ec.pem"

mkdir -p $(dirname "${PRIVATE_EC_CLIENT_KEY_PATH}")
openssl ecparam -name secp384r1 -genkey -noout -out "${PRIVATE_EC_CLIENT_KEY_PATH}" 2>/dev/null
openssl ec -in "${PRIVATE_EC_CLIENT_KEY_PATH}" -pubout -out "${PUBLIC_EC_CLIENT_KEY_PATH}" 2>/dev/null

FINGERPRINT=$(node scripts/js/fingerprint.js ${PUBLIC_EC_CLIENT_KEY_PATH})

PUBLIC_EC_CLIENT_KEY=$(cat "${PUBLIC_EC_CLIENT_KEY_PATH}" | sed -e '1d' -e '$d' | tr -d '\n')
PUBLIC_EC_CLIENT_KEY_URL_ENCODED=$(echo "${PUBLIC_EC_CLIENT_KEY}" | sed -e 's/+/-/g' -e 's/\//_/g' -e 's/=//g')
if [ "$(uname)" == "Darwin" ]; then
  open "${FRONTEND_HOST}/api/auth/device/register?spki=${PUBLIC_EC_CLIENT_KEY_URL_ENCODED}"
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
  xdg-open "${FRONTEND_HOST}/api/auth/device/register?spki=${PUBLIC_EC_CLIENT_KEY_URL_ENCODED}"
else 
    echo "Unsupported OS"
    exit 1
fi

echo "Registering key with id: $FINGERPRINT"
echo "Validate that the fingerprint matches the one shown in the browser :^)"

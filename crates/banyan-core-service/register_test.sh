#!/bin/bash

set -o errexit
set -o pipefail

AUTH_HOST="http://127.0.0.1:3000"

TMP_CERT_DIR="/tmp/ec_certs_gen"
PRIVATE_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/private.ec.key"
PUBLIC_EC_CLIENT_KEY_PATH="${TMP_CERT_DIR}/public.ec.pem"

mkdir -p $(dirname "${PRIVATE_EC_CLIENT_KEY_PATH}")
openssl ecparam -name secp384r1 -genkey -noout -out "${PRIVATE_EC_CLIENT_KEY_PATH}" 2>/dev/null
openssl ec -in "${PRIVATE_EC_CLIENT_KEY_PATH}" -pubout -out "${PUBLIC_EC_CLIENT_KEY_PATH}" 2>/dev/null

# TODO: extract the spki from the public key
# TODO: encode spki like this: base64str.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '.');

# TODO: open firefox to the registration page: firefox ${AUTH_HOST}/api/device/register?spki=MFkwEwYHKoZIzj0CAQYIKoZIzj0DA
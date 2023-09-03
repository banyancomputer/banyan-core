#!/bin/bash

set -o errexit
set -p nounset

EMAIL_ADDRESS="${1:-}"

if [ -z "${EMAIL_ADDRESS}" ]; then
  echo "Need to provide email address as first argument"
  exit 1
fi

# Note: You can use these scripts to add/remove users from the allow list
#       You should only have to add a user once, if using the postgres container in docker-compose.yml.
#       Make sure the email you allow is allowed through your OAuth2 provider.
# This endpoint only runs in development mode
curl -X POST \
  -H 'Content-Type: application/json' \
  -d "{ \"email\": \"${EMAIL_ADDRESS}\" }" \
 http://127.0.0.1:3000/api/admin/allow

# curl -X DELETE \
# -H "Content-Type: application/json" \
# -d '{ "email": "alex@banyan.computer" }' \
#  "http://localhost:3000/api/admin/allow"

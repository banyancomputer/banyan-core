#!/bin/bash

BASE_HOST="http://127.0.0.1:3000"

API_TOKEN="$(curl -s ${BASE_HOST}/api/v1/auth/fake_token)"

cat <<EOF | curl -s -H "Authorization: Bearer ${API_TOKEN}" -F "upload=@-;filename=my-notes.md" ${BASE_HOST}/api/v1/buckets/$(uuidgen)/publish
# Test file

This file was generated at $(date +%s.%N) or $(date).

$(fortune)
EOF
echo

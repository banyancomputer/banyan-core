#!/bin/bash

BASE_HOST="http://127.0.0.1:3000"
API_TOKEN="$(curl -s ${BASE_HOST}/api/v1/auth/fake_token)"
BUCKET_ID="$(uuidgen)"

cat <<EOF | curl -s -H "Authorization: Bearer ${API_TOKEN}" -H "Content-Type: application/vnd.ipld.car; version=2" --data-binary "@-" ${BASE_HOST}/api/v1/buckets/${BUCKET_ID}/publish
# Test file

This data file was generated at $(date +%s.%N) or $(date).

Your fortune (if available):

$(fortune 2>/dev/null || echo "No fortune for you...")
EOF
echo

#!/usr/bin/env bash

set -o errexit
set -o nounset

# For local development and production, we support Minio as an object storage service

# Name of the Minio container
MINIO_CONTAINER_NAME="banyan-minio"

# Name of the bucket used for staging
MINIO_STAGING_BUCKET_NAME="banyan-staging"

# Name of the bucket used for the storage provider
MINIO_STORAGE_PROVIDER_BUCKET_NAME="banyan-storage-provider"

# Use the crate's local data/ for mounting the volume
MINIO_VOLUME_MT_DIR="data"
MINIO_VOLUME_MT_PATH="$(pwd)/${MINIO_VOLUME_MT_DIR}"
MINIO_VOLUME_CONTAINER_PATH="/${MINIO_VOLUME_MT_DIR}"

# Credentials for the Minio API available at localhost:9000
# These can also be used as AWS credentials for the S3 API at localhost:9090
# This is fine for local development, but should be changed for production
MINIO_ROOT_USER="ROOTUSER"
MINIO_ROOT_PASSWORD="INSECURE"

CONTAINER_RUNTIME="podman"
if which docker &>/dev/null; then
	CONTAINER_RUNTIME="docker"
fi

# Safely start the Minio container
function run-minio {
	start-minio-container
}

# Create the Minio bucket for the staging service
function create-minio-staging-bucket {
	# Create a directory for the bucket on the host
	mkdir -p ${MINIO_VOLUME_MT_PATH}/${MINIO_STAGING_BUCKET_NAME}
	# Create the bucket in the Minio container
	${CONTAINER_RUNTIME} exec ${MINIO_CONTAINER_NAME} mc mb -p ${MINIO_VOLUME_CONTAINER_PATH}/${MINIO_STAGING_BUCKET_NAME}
}

# Create the Minio bucket for the storage provider service
function create-minio-storage-provider-bucket {
	# Create a directory for the bucket on the host
	mkdir -p ${MINIO_VOLUME_MT_PATH}/${MINIO_STORAGE_PROVIDER_BUCKET_NAME}
	# Create the bucket in the Minio container
	${CONTAINER_RUNTIME} exec ${MINIO_CONTAINER_NAME} mc mb -p ${MINIO_VOLUME_CONTAINER_PATH}/${MINIO_STORAGE_PROVIDER_BUCKET_NAME}
}

# Helpers:

# Start the Minio container, making sure it exists on the host
function start-minio-container {
	ensure-minio-container-exists
	${CONTAINER_RUNTIME} start ${MINIO_CONTAINER_NAME}
}

# Create the Minio container if it does not exist
function ensure-minio-container-exists {
	create-minio-container
}

# Create the Minio container
function create-minio-container {
	# Create the Minio container with the Minio volume mounted
	${CONTAINER_RUNTIME} run \
		-p 9000:9000 \
		-p 9090:9090 \
		--name ${MINIO_CONTAINER_NAME} --rm -d \
		-e "MINIO_ROOT_USER=${MINIO_ROOT_USER}" \
		-e "MINIO_ROOT_PASSWORD=${MINIO_ROOT_PASSWORD}" \
		-v banyan-minio-data:${MINIO_VOLUME_CONTAINER_PATH} \
		quay.io/minio/minio server ${MINIO_VOLUME_CONTAINER_PATH} --console-address ":9090"
}

function clean() {
	${CONTAINER_RUNTIME} rm -fv ${MINIO_CONTAINER_NAME} || true
}

$1

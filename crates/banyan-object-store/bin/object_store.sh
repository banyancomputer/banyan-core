#!/usr/bin/env bash

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

# These are the paths to the buckets on the Minio volume for our services
MINIO_STAGING_BUCKET_MT_PATH="/${MINIO_VOLUME_MT_DIR}/${MINIO_STAGING_BUCKET_NAME}"
MINIO_STORAGE_PROVIDER_BUCKET_MT_PATH="/${MINIO_VOLUME_MT_DIR}/${MINIO_STORAGE_PROVIDER_BUCKET_NAME}"

# Credentials for the Minio API available at localhost:9000
# These can also be used as AWS credentials for the S3 API at localhost:9090
# This is fine for local development, but should be changed for production
MINIO_ROOT_USER="ROOTUSER"
MINIO_ROOT_PASSWORD="INSECURE"

# Safely start the Minio container
function run-minio {
	start-minio-container
}

# Create the Minio bucket for the staging service
function create-minio-staging-bucket {
	docker exec ${MINIO_CONTAINER_NAME} mc mb -p ${MINIO_STAGING_BUCKET_MT_PATH}
}

# Create the Minio bucket for the storage provider service
function create-minio-storage-provider-bucket {
	docker exec ${MINIO_CONTAINER_NAME} mc mb -p ${MINIO_STORAGE_PROVIDER_BUCKET_MT_PATH}
}

# Helpers:

# Start the Minio container, making sure it exists on the host
function start-minio-container {
	insure-minio-container-exists
	docker start ${MINIO_CONTAINER_NAME}
}

# Create the Minio container if it does not exist
function insure-minio-container-exists {
	if [ -z "$(docker ps -a -q -f name=${MINIO_CONTAINER_NAME})" ]; then
		create-minio-container
	fi
}

# Create the Minio container
function create-minio-container {
	mkdir -p ${MINIO_VOLUME_MT_PATH}
	# Create the Minio container with the Minio volume mounted
	docker create \
		-p 9000:9000 \
		-p 9090:9090 \
		--user $(id -u):$(id -g) \
		--name ${MINIO_CONTAINER_NAME} \
		-e "MINIO_ROOT_USER=${MINIO_ROOT_USER}" \
		-e "MINIO_ROOT_PASSWORD=${MINIO_ROOT_PASSWORD}" \
		-v ${MINIO_VOLUME_MT_PATH}:/${MINIO_VOLUME_MT_DIR} \
		quay.io/minio/minio server /${MINIO_VOLUME_MT_DIR} --console-address ":9090"
}

$1 $2
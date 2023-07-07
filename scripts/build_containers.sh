#!/bin/bash

set -o errexit
set -o errtrace
set -o noclobber
set -o nounset
set -o pipefail

function error_handler() {
	local error_line=$1
	local status_code=$2

	local pipe_status=($3)
	local funcs=($4)

	echo -n "Error occurred executing line ${error_line} with status code ${status_code}"

	if [ "${#pipe_status}" -gt 1 ]; then
		echo -n " (pipe codes: ${pipe_status[@]})"
	fi
}

if ! trap -p | grep -q error_handler; then
	trap 'error_handler ${LINENO} $? "${PIPESTATUS[*]}" "${FUNCNAME[*]}"' ERR
fi

# Collect git specific information
GIT_REF=$(git describe --always --dirty --long --tags)
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

SERVICES=("banyan-core-service" "banyan-staging-service" "banyan-storage-provider-service")

# Prefer podman where available
CONTAINER_RUNTIME="docker"
if which -q podman &>/dev/null; then
	CONTAINER_RUNTIME="podman"
fi

for SERVICE in ${SERVICES[@]}; do
	if [ -n "${GCR_PROJECT:-}" ]; then
		CONTAINER_NAME="gcr.io/${GCR_PROJECT}/${SERVICE}"
	else
		CONTAINER_NAME="${SERVICE}"
	fi

	echo -n "Building ${CONTAINER_NAME}... "
	BUILD_OUTPUT="$(${CONTAINER_RUNTIME} build -t ${CONTAINER_NAME}:${GIT_REF} --build-arg SERVICE=${SERVICE} . 2>&1)"
	if [ ! $? ]; then
		echo -e "FAILED\nBuild failed with the following outputs:\n${BUILD_OUTPUT}"
		exit 1
	fi
	echo "Success"

	echo "Container can be used with the following name/tags:"
	echo -e "\t- ${CONTAINER_NAME}:${GIT_REF}"

	if [ "${CURRENT_BRANCH}" == "main" ]; then
		${CONTAINER_RUNTIME} tag ${CONTAINER_NAME}:${GIT_REF} ${CONTAINER_NAME}:latest &>/dev/null
		echo -e "\t- ${CONTAINER_NAME}:latest"
	fi
done

# If the GCR variables are setup, push the container to the remote repo.
if [ -n "${GCLOUD_SERVICE_KEY:-}" -a -n "${GCR_PROJECT:-}" ]; then
	echo ${GCLOUD_SERVICE_KEY} | ${CONTAINER_RUNTIME} login -u _json_key --password-stdin https://gcr.io/${GCR_PROJECT}

	for SERVICE in ${SERVICES[@]}; do
		${CONTAINER_RUNTIME} push "gcr.io/${GCR_PROJECT}/${SERVICE}"
	done
else
	echo 'WARNING: Missing environment configs required to push container.'
fi

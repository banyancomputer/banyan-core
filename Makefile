# NB: the project will be built if make is invoked without any arguments.
.PHONY: default
default: build

.PHONY: build
build:
	cargo build --workspace

.PHONY: check
check:
	cargo check --workspace

.PHONY: clean
clean:
	rm -rf \
		crates/banyan-core-service/data/serv* \
		crates/banyan-core-service/data/uploads/*
	rm -rf crates/banyan-staging-service/data/serv* \
		crates/banyan-staging-service/data/platform* \
		crates/banyan-staging-service/data/uploads/*
	rm -rf crates/banyan-storage-provider-service/data/serv* \
		crates/banyan-storage-provider-service/data/platform* \
		crates/banyan-storage-provider-service/data/uploads/* 

.PHONY: fmt
fmt:
	cargo +nightly fmt --all

.PHONY: fmt-check
fmt-check:
	cargo +nightly fmt --all -- --check

# generate authentication keys
# =========================================================================== #
# TODO: these commands generate a key by starting a service, and terminating
# them after 3 seconds. eventually, we could find a way to do this that does
# not risk flaking. for now, check that the key material exists by examining
# the services' respective `data/` directories.

.PHONY: generate-core-service-key
generate-core-service-key:
	(                                         \
		cd crates/banyan-core-service     \
		&& cargo build                    \
		&& (timeout 3s cargo run || true) \
	)

.PHONY: generate-staging-service-key
generate-staging-service-key:
	(                                         \
		cd crates/banyan-staging-service  \
		&& cargo build                    \
		&& (timeout 3s cargo run || true) \
	)

.PHONY: generate-storage-provider-service-key
generate-storage-provider-service-key:
	(                                                 \
		cd crates/banyan-storage-provider-service \
		&& cargo build                            \
		&& (timeout 3s cargo run || true)         \
	)

# database administration
# =========================================================================== #

.PHONY: connect-to-core-database
connect-to-core-database:
	sqlite3 crates/banyan-core-service/data/server.db

.PHONY: connect-to-staging-database
connect-to-staging-database:
	sqlite3 crates/banyan-staging-service/data/server.db

.PHONY: connect-to-storage-provider-database
connect-to-storage-provider-database:
	sqlite3 crates/banyan-storage-provider-service/data/server.db

# object storage administration
# =========================================================================== #

# for local development and production, we support minio as an object storage service

# name of the minio container
minio_container_name = banyan-minio
# name of the bucket used for staging
minio_staging_bucket_name = banyan-staging
# name of the bucket used for the storage provider
minio_storage_provider_bucket_name = banyan-storage-provider
# path to the minio volume on the container
minio_volume_mt_dir_name = data
# where the minio volume is mounted on the host 
minio_volume_path = ${HOME}/$(minio_container_name)/$(minio_volume_mt_dir_name)

# these are the paths to the buckets on the minio volume for our services
minio_staging_bucket_mt_path = /$(minio_volume_mt_dir_name)/$(minio_staging_bucket_name)
minio_storage_provider_bucket_mt_path = /$(minio_volume_mt_dir_name)/$(minio_storage_provider_bucket_name)

# credentials for the minio API available at localhost:9000
# these can also be used as aws credentials for the s3 API at localhost:9090
# this is fine for local development, but should be changed for production
minio_root_user = ROOTUSER
minio_root_password = INSECURE

.PHONY: run-minio \
	stop-minio \
	rm-minio-volume \
	create-minio-staging-bucket \
	create-minio-storage-provider-bucket \
	create-minio-volume \
	insure-minio-container-exists \
	create-minio-container start-minio-container

# safely start the minio container
run-minio: start-minio-container

# stop the minio container
stop-minio: stop-minio-container

# remove the minio volume
rm-minio-volume: 
	rm -rf ${minio_volume_path}

# create the minio bucket for the staging service
create-minio-staging-bucket:
	docker exec ${minio_container_name} mc mb ${minio_staging_bucket_mt_path} 

# create the minio bucket for the storage provider service
create-minio-storage-provider-bucket:
	docker exec ${minio_container_name} mc mb ${minio_storage_provider_bucket_mt_path}

# Helpers:

# create the minio volume
create-minio-volume:
	mkdir -p ${minio_volume_path}

# start the minio container, making sure it exists on the host
start-minio-container: insure-minio-container-exists
	docker start ${minio_container_name}

# stop the minio container -- nothing special here
stop-minio-container:
	docker stop ${minio_container_name}

# create the minio container if it does not exist
insure-minio-container-exists:
	@if [ -z "$(docker ps -a -q -f name=${minio_container_name})" ]; then \
		echo "Creating minio container..."; \
		$(MAKE) create-minio-container; \
	fi

# create the minio container
create-minio-container: create-minio-volume
	docker create \
		-p 9000:9000 \
		-p 9090:9090 \
		--user $(shell id -u):$(shell id -g) \
		--name ${minio_container_name} \
		-e "MINIO_ROOT_USER=$(minio_root_user)" \
		-e "MINIO_ROOT_PASSWORD=$(minio_root_password)" \
		-v ${minio_volume_path}:/${minio_volume_mt_dir_name} \
		quay.io/minio/minio server /${minio_volume_mt_dir_name} --console-address ":9090"
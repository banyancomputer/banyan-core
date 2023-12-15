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
		crates/banyan-storage-provider-service/data/uploads/* \
	rm -rf ${minio_staging_bucket_path} \
		${minio_storage_provider_bucket_path}

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

# name of the minio container
minio_container_name = banyan-minio
# path to the minio volume on the container
minio_volume_mt_dir_name = data
# where the minio volume is mounted on the host 
minio_volume_path = ${HOME}/$(minio_container_name)/$(minio_volume_mt_dir_name)
# name of the staging service's minio bucket
minio_staging_bucket_name = banyan-staging
# name of the storage provider service's minio bucket
minio_storage_provider_bucket_name = banyan-storage-provider

# paths to the buckets on the minio container
minio_staging_bucket_path = $(minio_volume_path)/$(minio_staging_bucket_name)
minio_storage_provider_bucket_path = $(minio_volume_path)/$(minio_storage_provider_bucket_name)
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
	rm -rf crates/banyan-object-store/data/banyan-staging \
		crates/banyan-object-store/data/banyan-storage-provider
	./crates/banyan-object-store/bin/object_store.sh clean

.PHONY: fmt
fmt:
	cargo +nightly fmt --all

.PHONY: fmt-check
fmt-check:
	cargo +nightly fmt --all -- --check

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets --all-features --tests -- -D warnings

.PHONY: minio
minio:
	./crates/banyan-object-store/bin/object_store.sh run-minio
	./crates/banyan-object-store/bin/object_store.sh create-minio-staging-bucket
	./crates/banyan-object-store/bin/object_store.sh create-minio-storage-provider-bucket

.PHONY: test
test:
	cargo test --all --workspace --bins --tests --benches

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

crates/banyan-staging-service/data/uploads:
	mkdir -p crates/banyan-staging-service/data/uploads

.PHONY: generate-staging-service-key
generate-staging-service-key: crates/banyan-staging-service/data/uploads
	(                                         \
		cd crates/banyan-staging-service  \
		&& cargo build                    \
		&& (timeout 3s cargo run || true) \
	)

crates/banyan-storage-provider-service/data/uploads:
	mkdir -p crates/banyan-storage-provider-service/data/uploads

.PHONY: generate-storage-provider-service-key
generate-storage-provider-service-key: crates/banyan-storage-provider-service/data/uploads
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

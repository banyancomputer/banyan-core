# NB: the project will be built if make is invoked without any arguments.
.PHONY: default
default: build

.PHONY: fmt
fmt:
	cargo fmt -p banyan-core-service banyan-staging-service banyan-storage-provider-service banyan-middleware

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

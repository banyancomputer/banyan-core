#!/usr/bin/make -f

IS_TTY:=$(shell (test -t 0 && echo yes) || echo no)

ifeq ($(IS_TTY),yes)
ANSI_RESET:=setterm --reset --default
ANSI_FG_BOLD_RED:=setterm --bold on --foreground red
else
ANSI_RESET:=true
ANSI_FG_BOLD_RED:=true
endif

GITHUB_ORG:=banyancomputer
GITHUB_REPO:=banyan-core
GITHUB_URL:=https://github.com/$(GITHUB_ORG)/$(GITHUB_REPO)

GIT_COMMIT:=$(shell git rev-parse 2>/dev/null HEAD || echo unknown)
GIT_COMMIT_SHORT:=$(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
GIT_DESC:=$(shell git describe --always --dirty --long --tags 2>/dev/null || echo unknown)

DOCKER_REGISTRY_URI=ghcr.io
DOCKER_ORG=banyancomputer

DOCKER_CACHE?=y

ifeq ($(DOCKER_CACHE),y)
DOCKER_CACHE_ARG:=
else
DOCKER_CACHE_ARG:=--no-cache
endif

# NB: the project will be built if make is invoked without any arguments.
.PHONY: default
default: build

.PHONY: build
build:
	cargo build --workspace

.PHONY: check
check:
	cargo check --workspace --all-targets --all-features --tests

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

FRONTEND?=frontend

.PHONY: service-test frontend-install frontend-build frontend-test frontend-audit frontend-clean
service-test:
	@# set SERVICE to the desired service to build
	@if [ -z "$${SERVICE}" ]; then \
  		( \
  			$(ANSI_FG_BOLD_RED) ; \
  			echo 'ERROR: please set SERVICE to the name of the service to produce' ; \
  			echo '       e.g. `banyan-core-service` or `banyan-staging-service`' ; \
  			$(ANSI_RESET) ; \
  		) >&2 ; \
		exit 1 ; \
	fi

frontend-install: service-test
	(cd "crates/$${SERVICE}/$(FRONTEND)" && yarn install)

frontend-build: service-test
	(cd "crates/$${SERVICE}/$(FRONTEND)" && yarn build)

frontend-test: service-test
	(cd "crates/$${SERVICE}/$(FRONTEND)" && yarn run test)

frontend-audit: service-test
	(cd "crates/$${SERVICE}/$(FRONTEND)" && yarn audit)

frontend-clean:
	git clean -d -x -f \
		crates/banyan-core-service/admin_dist \
		crates/banyan-core-service/dist \
		crates/banyan-core-service/admin_frontend \
		crates/banyan-core-service/frontend \
		crates/banyan-storage-provider-service/dist \
		crates/banyan-storage-provider-service/frontend

.PHONY: backend-clean
backend-clean:
	cargo clean

.PHONY: yarn-clean
yarn-clean:
	rm -fr "~/.cache/yarn"

.PHONY: docker
docker: service-test
	@# on linux, run docker buildx install once per machine to enable buildx as the default builder
	docker buildx build --progress plain --file ./Dockerfile \
		$(DOCKER_CACHE_ARG) \
		--build-arg "SERVICE=$(SERVICE)" \
		--build-arg "GIT_COMMIT=$(GIT_COMMIT)" \
		--build-arg "CI_BUILD_REF=$(GIT_DESC)" \
		--label "org.opencontainers.image.created=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)" \
		--label "org.opencontainers.image.revision=$(GIT_COMMIT)" \
		--label "org.opencontainers.image.source=$(GITHUB_URL)" \
		--label "org.opencontainers.image.title=$(SERVICE)" \
		--label "org.opencontainers.image.url=$(GITHUB_URL)" \
		--label "org.opencontainers.image.version=$(GIT_COMMIT)" \
		--label "org.opencontainers.image.build-env=local" \
		-t "$(DOCKER_REGISTRY_URI)/$(DOCKER_ORG)/$${SERVICE}:unstable" \
		-t "$(DOCKER_REGISTRY_URI)/$(DOCKER_ORG)/$${SERVICE}:$(GIT_COMMIT)" \
		-t "$(DOCKER_REGISTRY_URI)/$(DOCKER_ORG)/$${SERVICE}:$(GIT_COMMIT_SHORT)" \
		.
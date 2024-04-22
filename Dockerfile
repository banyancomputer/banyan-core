# cache-busting: change this value to invalidate a cache completely
ARG CACHE_VERSION=v1
# ids for the various caches
ARG CARGO_BUILD_CACHE_ID=com.github.banyancomputer.banyan-core.cargo-build.${CACHE_VERSION}
ARG CARGO_REGISTRY_CACHE_ID=com.github.banyancomputer.banyan-core.cargo-registry.${CACHE_VERSION}
ARG SCCACHE_CACHE_ID=com.github.banyancomputer.banyan-core.sccache.${CACHE_VERSION}
ARG CORE_SVC_NODE_CACHE_ID=com.github.banyancomputer.banyan-core.core-service.node.${CACHE_VERSION}
ARG CORE_SVC_ADMIN_NODE_CACHE_ID=com.github.banyancomputer.banyan-core.core-service.node-admin.${CACHE_VERSION}
ARG STORAGE_SVC_NODE_CACHE_ID=com.github.banyancomputer.banyan-core.storage-service.node.${CACHE_VERSION}

# the version of the rust image to use
ARG RUST_VERSION=1.77
# the image for the rust builder phase
ARG RUST_BUILDER_IMAGE=rust:${RUST_VERSION}-alpine

# the version of the node image to use
ARG NODE_VERSION=20.11
# the image for the node builder phase
ARG NODE_BUILDER_IMAGE=node:${NODE_VERSION}-alpine

# the version of the runtime (alpine) image to use
ARG RUNTIME_IMAGE_VERSION=latest
# the image for the runtime phase
ARG RUNTIME_IMAGE=alpine:${RUNTIME_IMAGE_VERSION}

# the service to build
ARG SERVICE
# the commit/release marker, generated with `git describe --always --dirty --long --tags`
ARG CI_BUILD_REF
ARG GIT_COMMIT

# rust builder phase ---------------------------------------------------------------------------------------------------
FROM ${RUST_BUILDER_IMAGE} AS rust-build

# import
ARG RUST_VERSION
ARG SERVICE

# sanity test
RUN if [ -z "$SERVICE" ]; then \
        ( echo 'ERROR: please specify the SERVICE build arg' ) >&2 ; \
        exit 1 ; \
    fi

# install os packages
RUN apk add --virtual build-dependencies curl tree build-base openssl openssl-dev perl mold binutils

# install cargo-binstall; bump the env variable to purge the cache
ENV BUILD_UTILS_CACHE_VERSION=v1
RUN curl -fsSL "https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz" | \
        tar xzf - && install -m 0755 ./cargo-binstall /usr/local/bin/ && rm ./cargo-binstall

# install utilities
RUN cargo binstall -y cargo-auditable cargo-audit sccache

# setup app build dir
RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app


COPY ./Cargo.toml ./Cargo.lock rust-toolchain.toml deny.toml ./
COPY crates/banyan-api-client/Cargo.toml               ./crates/banyan-api-client/
COPY crates/banyan-car-analyzer/Cargo.toml             ./crates/banyan-car-analyzer/
COPY crates/banyan-core-service/Cargo.toml             ./crates/banyan-core-service/
COPY crates/banyan-object-store/Cargo.toml             ./crates/banyan-object-store/
COPY crates/banyan-staging-service/Cargo.toml          ./crates/banyan-staging-service/
COPY crates/banyan-storage-provider-service/Cargo.toml ./crates/banyan-storage-provider-service/
COPY crates/banyan-task/Cargo.toml                     ./crates/banyan-task/
COPY crates/banyan-traffic-counter/Cargo.toml          ./crates/banyan-traffic-counter/

# emit metadata about the local environment
RUN find ./ -type f \( -name 'Cargo.toml' -or -name 'Cargo.lock' \) | sort | xargs sha256sum && \
    tree -a .

# import
ARG CARGO_BUILD_CACHE_ID
ARG CARGO_REGISTRY_CACHE_ID

# download dependencies
RUN --mount=type=cache,id=${CARGO_BUILD_CACHE_ID},target=/usr/src/app/target \
    --mount=type=cache,id=${CARGO_REGISTRY_CACHE_ID},target=/usr/local/cargo/registry \
    cargo fetch --manifest-path ./Cargo.toml

# source step
COPY crates/banyan-api-client/               ./crates/banyan-api-client/
COPY crates/banyan-car-analyzer/             ./crates/banyan-car-analyzer/
COPY crates/banyan-core-service/             ./crates/banyan-core-service/
COPY crates/banyan-object-store/             ./crates/banyan-object-store/
COPY crates/banyan-staging-service/          ./crates/banyan-staging-service/
COPY crates/banyan-storage-provider-service/ ./crates/banyan-storage-provider-service/
COPY crates/banyan-task/                     ./crates/banyan-task/
COPY crates/banyan-traffic-counter/          ./crates/banyan-traffic-counter/

# emit metadata
RUN tree -aL 3 ./

# create dist if needed
RUN mkdir -p ./crates/$SERVICE/dist

# import
ARG SCCACHE_CACHE_ID
ARG GIT_COMMIT
ARG CI_BUILD_REF

# build it
RUN --mount=type=cache,id=${CARGO_BUILD_CACHE_ID},target=/usr/src/app/target \
    --mount=type=cache,id=${CARGO_REGISTRY_CACHE_ID},target=/usr/local/cargo/registry \
    --mount=type=cache,id=${SCCACHE_CACHE_ID},target=/root/.cache/sccache \
    RUSTC_WRAPPER=sccache CI_BUILD_REF=$CI_BUILD_REF GIT_COMMIT=$GIT_COMMIT \
      mold -run cargo auditable build --target x86_64-unknown-linux-musl --bin $SERVICE --release && \
    du -bh target/x86_64-unknown-linux-musl/release/$SERVICE && \
    cp target/x86_64-unknown-linux-musl/release/$SERVICE ./

# verify we linked with mold
RUN readelf -p .comment ./$SERVICE
RUN if ! readelf -p .comment ./$SERVICE | grep -q '\bmold\b' ; then \
        echo 'ERROR: Final binary not linked with mold' >&2 ; \
        exit 1 ; \
    fi

# node builder phase ---------------------------------------------------------------------------------------------------
FROM ${NODE_BUILDER_IMAGE} as node-build

# setup app build dir
RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

# dependency step: copy in package.json and package-lock.json
COPY crates/banyan-core-service/frontend/package.json crates/banyan-core-service/frontend/package-lock.json ./crates/banyan-core-service/frontend/
COPY crates/banyan-core-service/admin_frontend/package.json crates/banyan-core-service/admin_frontend/package-lock.json ./crates/banyan-core-service/admin_frontend/
COPY crates/banyan-storage-provider-service/frontend/package.json crates/banyan-storage-provider-service/frontend/package-lock.json ./crates/banyan-storage-provider-service/frontend/

# NOTE we must also copy in banyan-core-service/frontend/tomb_build
COPY crates/banyan-core-service/frontend/tomb_build/ crates/banyan-core-service/frontend/tomb_build/

# import
ARG CORE_SVC_NODE_CACHE_ID
ARG STORAGE_SVC_NODE_CACHE_ID
ARG SERVICE

# [DOWNLOAD] download dependencies for front-end
RUN --mount=type=cache,id=${CORE_SVC_NODE_CACHE_ID},target=/usr/src/app/crates/banyan-core-service/frontend/node_modules \
    --mount=type=cache,id=${STORAGE_SVC_NODE_CACHE_ID},target=/usr/src/app/crates/banyan-storage-provider-service/frontend/node_modules \
    if [ -d "crates/$SERVICE/frontend" ]; then \
        echo 'INFO: Fetching front-end dependencies' >&2 ; \
        ( cd "crates/$SERVICE/frontend" && npm install ); \
    else \
        echo 'INFO: No front-end to fetch dependencies for' >&2 ; \
    fi

# import
ARG CORE_SVC_NODE_ADMIN_CACHE_ID

# [DOWNLOAD] download dependencies for admin front-end
RUN --mount=type=cache,id=${CORE_SVC_NODE_ADMIN_CACHE_ID},target=/usr/src/app/crates/banyan-core-service/admin_frontend/node_modules \
    if [ -d "crates/$SERVICE/admin_frontend" ]; then \
        echo 'INFO: Fetching admin front-end dependencies' >&2 ; \
        ( cd "crates/$SERVICE/admin_frontend" && npm install ) ; \
    else \
        echo 'INFO: No admin front-end to fetch dependencies for' >&2 ; \
    fi

# copy in code
COPY crates/banyan-core-service/admin_frontend/       ./crates/banyan-core-service/admin_frontend/
COPY crates/banyan-core-service/frontend/             ./crates/banyan-core-service/frontend/
COPY crates/banyan-storage-provider-service/frontend/ ./crates/banyan-storage-provider-service/frontend/

# [BUILD] build the front-end if it exists and place it in the root of the repo at ./dist
RUN --mount=type=cache,id=${CORE_SVC_NODE_CACHE_ID},target=/usr/src/app/crates/banyan-core-service/frontend/node_modules \
    --mount=type=cache,id=${STORAGE_SVC_NODE_CACHE_ID},target=/usr/src/app/crates/banyan-storage-provider-service/frontend/node_modules \
    if [ -d  "crates/$SERVICE/frontend" ]; then \
      echo 'INFO: Building front-end' >&2 ; \
      ( cd "crates/$SERVICE/frontend" && npm run build ) && cp -r "crates/$SERVICE/dist" ./ ; \
    else \
      echo 'INFO: Service has no front-end to build' ; \
      mkdir -p ./dist ; \
    fi

# [BUILD] build the admin front-end if it exists and place it in the root of the repo at ./admin_dist
RUN --mount=type=cache,id=${CORE_SVC_NODE_ADMIN_CACHE_ID},target=/usr/src/app/crates/banyan-core-service/admin_frontend/node_modules \
    if [ -d  "crates/$SERVICE/admin_frontend" ]; then \
      echo 'INFO: Building admin front-end' >&2 ; \
      ( cd "crates/$SERVICE/admin_frontend" && npm run build ) && cp -r "crates/$SERVICE/admin_dist" ./ ; \
    else \
      echo 'INFO: Service has no front-end to build' >&2 ; \
      mkdir -p ./admin_dist ; \
    fi

# sanity testing
RUN if [ ! -d "./dist" ]; then \
        echo 'ERROR: dist folder was not created/generated' >&2 && exit 1 ; \
    fi ; \
    if [ ! -d "./admin_dist" ]; then \
        echo 'ERROR: admin_dist folder was not created/generated' >&2 && exit 1 ; \
    fi

# runtime phase --------------------------------------------------------------------------------------------------------
FROM ${RUNTIME_IMAGE} as runtime

# import build args
ARG SERVICE

# bring in the service binary
COPY --from=rust-build /usr/src/app/$SERVICE /usr/bin/service

# bring in migrations and web front-end
COPY --from=node-build /usr/src/app/dist /svc/root/dist
COPY --from=node-build /usr/src/app/admin_dist /svc/root/admin_dist
COPY --from=rust-build /usr/src/app/crates/$SERVICE/migrations /svc/root/migrations

# create the runtime user and home dir
RUN adduser -D -h /svc/root runtime

# set ownership
RUN chown -R runtime:runtime /svc/root/ /usr/bin/service

# drop permissions
USER runtime

# set pwd
WORKDIR /svc/root

VOLUME /data

ENTRYPOINT ["/usr/bin/service"]
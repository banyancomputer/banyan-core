# Do most of our build in one container, we don't need the intermediate
# artifacts or build requirements in our release container. We'll copy in our
# produced binary to the final production container later.
FROM docker.io/library/rust:1.71.0 AS build

ARG CI_BUILD_REF=
ARG FEATURES=sqlite
ARG SERVICE_NAME=banyan-staging-service

RUN mkdir -p /usr/src/build
WORKDIR /usr/src/build

# Perform a release build only using the dependencies and an otherwise empty
# binary project, to allow the dependencies to build and be cached. This
# prevents rebuilding them in the future if only the service's source has
# changed.
RUN cargo init --name $SERVICE_NAME
COPY Cargo.toml Cargo.lock /usr/src/build
RUN cargo build --release --no-default-features --features $FEATURES

# Copy in the actual service source code, and perform the release build
# (install is release mode by default).
COPY build.rs /usr/src/build/build.rs
COPY migrations /usr/src/build/migrations
COPY src /usr/src/build/src

# The container build environment doesn't have access to the entire git repo
# only the immediate sources. We get around this by passing it into the build
# system from outside.
ENV CI_BUILD_REF=$CI_BUILD_REF

RUN cargo install --bins --path ./
RUN strip --strip-unneeded /usr/local/cargo/bin/$SERVICE_NAME
RUN mv /usr/local/cargo/bin/$SERVICE_NAME /usr/local/cargo/bin/service

# Use an absolutely minimal container with the barest permissions to limit
# sources of security vulnerabilities, and ensure that any security issues are
# extremely scoped in how they can be exploited.
FROM gcr.io/distroless/cc-debian11:debug-nonroot

# Bring in just our final compiled artifact
COPY --from=build /usr/local/cargo/bin/service /usr/bin/service

VOLUME /data

ENTRYPOINT ["/usr/bin/service"]

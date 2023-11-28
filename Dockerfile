# Do most of our build in one container, we don't need the intermediate
# artifacts or build requirements in our release container. We'll copy in our
# produced binary to the final production container later.
FROM docker.io/library/rust:1.71.0 AS build

ARG CI_BUILD_REF=development-container
ARG CRATE_NAME

RUN test -n "$CRATE_NAME" || { echo "Crate name must be passed to container build"; exit 1; }

RUN mkdir -p /usr/src/build
COPY . /usr/src/build

# The container build environment doesn't have access to the entire git repo
# only the immediate sources. We get around this by passing it into the build
# system from outside.
ENV CI_BUILD_REF=$CI_BUILD_REF

WORKDIR /usr/src/build/crates/$CRATE_NAME

RUN cargo install --bin $CRATE_NAME --path ./
RUN strip --strip-unneeded /usr/local/cargo/bin/$CRATE_NAME
RUN mv /usr/local/cargo/bin/$CRATE_NAME /usr/local/cargo/bin/service

# Use an absolutely minimal container with the barest permissions to limit
# sources of security vulnerabilities, and ensure that any security issues are
# extremely scoped in how they can be exploited.
FROM gcr.io/distroless/cc-debian11:nonroot

ARG CRATE_NAME

# Bring in just our final compiled artifact
COPY --from=build /usr/local/cargo/bin/service /usr/bin/service

COPY --from=build /usr/src/build/crates/$CRATE_NAME/migrations /svc/root/migrations
WORKDIR /svc/root

VOLUME /data

ENTRYPOINT ["/usr/bin/service"]

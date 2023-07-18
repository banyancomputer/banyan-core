# Do most of our build in one container, we don't need the intermediate
# artifacts or build requirements in our release container. We'll copy in our
# produced binary to the final production container later.
FROM docker.io/library/rust:1.70.0 AS build

RUN mkdir -p /usr/src/build/crates
WORKDIR /usr/src/build

# This must be specified, the run command here provides friendly errors as a
# bandaid over the inability to mark an argument as required.
ARG SERVICE
RUN if [ -z "$SERVICE" ]; then echo 'ERROR: container build requires SERVICE argument to be provided wiht the --build-arg flag.' && exit 1; fi

# Perform a release build only using the dependencies and an otherwise empty
# binary project, to allow the dependencies to build and be cached. This
# prevents rebuilding them in the future if only the service's source has
# changed.
#
# If additional repository local crates are needed in the future (such as
# library crates), those will need to be added via new COPY commands before the
# build command.
COPY Cargo.toml Cargo.lock ./
RUN cargo new crates/$SERVICE
COPY crates/$SERVICE/Cargo.toml /usr/src/build/crates/$SERVICE/
RUN cargo build -p $SERVICE --release

# Copy in the actual service source code, and perform the release build
# (install is release mode by default).
COPY crates/$SERVICE/build.rs /usr/src/build/crates/$SERVICE/build.rs
COPY crates/$SERVICE/migrations /usr/src/build/crates/$SERVICE/migrations
COPY crates/$SERVICE/src /usr/src/build/crates/$SERVICE/src
RUN cargo install --path crates/$SERVICE/ --bins

# Use an absolutely minimal container with the barest permissions to limit
# sources of security vulnerabilities, and ensure that any security issues are
# extremely scoped in how they can be exploited.
FROM gcr.io/distroless/cc-debian11:nonroot

# Bring in just our final compiled artifact
COPY --from=build /usr/local/cargo/bin/$SERVICE /usr/bin/$SERVICE

EXPOSE 3000
VOLUME /data

CMD ["/usr/bin/$SERVICE"]

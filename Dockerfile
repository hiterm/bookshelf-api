# https://docs.docker.com/language/rust/develop/

FROM rust:1.96.1-trixie@sha256:1f0dbad1df66647807e6952d1db85d0b2bda7606cb2139d82517e4f009967376 AS build-stage

ARG APP_NAME=bookshelf-api

ARG BUILDDIR=/app
WORKDIR ${BUILDDIR}

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=e2e,target=e2e \
    --mount=type=cache,target=${BUILDDIR}/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=bind,source=migrations,target=migrations \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
cp ./target/release/check_tls /bin/check_tls
EOF


# Shared base for the production image and the TLS regression-test image.
# Both stages inherit ca-certificates from here, ensuring that the regression
# test (tls-check) exercises the exact same certificate environment as
# production. Removing ca-certificates from this stage breaks both.
FROM debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd AS base

# https://ianwwagner.com/reqwest-0-13-upgrade-and-webpki.html
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/build/building/best-practices/#user
ARG UID=10001
RUN useradd -l -M -u "${UID}" -d "/nonexistent" -s "/sbin/nologin" appuser
USER appuser


# Regression test image for the CA certificate fix (PR #187).
# Verifies that reqwest can establish an HTTPS connection using the system
# trust store inherited from the base stage. Placed before the production
# stage so that the production image remains the default build target.
# Usage: docker build --target tls-check -t bookshelf-api:tls-check .
#        docker run --rm bookshelf-api:tls-check
FROM base AS tls-check

COPY --from=build-stage /bin/check_tls /bin/

CMD ["/bin/check_tls"]


# Production image — default build target (must be the last stage).
FROM base

COPY --from=build-stage /bin/server /bin/

CMD ["/bin/server"]

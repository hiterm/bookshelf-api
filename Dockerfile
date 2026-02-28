# https://docs.docker.com/language/rust/develop/

FROM rust:1.93.1-trixie AS build-stage

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
EOF


FROM debian:trixie-slim

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best_practices/   #user
ARG UID=10001
RUN useradd -M -u "${UID}" -d "/nonexistent" -s "/sbin/nologin" appuser
USER appuser

COPY --from=build-stage /bin/server /bin/

CMD ["/bin/server"]

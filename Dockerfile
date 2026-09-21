# https://docs.docker.com/language/rust/develop/

FROM rust:1.98.0-trixie@sha256:7f7a53a25a0319dd8284e279d529d45759cb384d59b14cc6806132910f45522e AS build-stage

ARG APP_NAME=bookshelf-api
ARG SCCACHE_ENABLED=false
ARG TARGETARCH

ARG SCCACHE_VERSION=0.17.0
RUN <<EOF
set -e
case "${TARGETARCH}" in
    amd64)
        sccache_arch=x86_64
        sccache_sha256=67c4a96dd237c1f518f6b36083f270f9976d516f1e57fce891755ea782e50006
        ;;
    arm64)
        sccache_arch=aarch64
        sccache_sha256=821a86343191aa1cbab74bd42f9e93c9a63bf85e4742945f40d3ae84193c1c77
        ;;
    *)
        echo "Unsupported architecture for sccache: ${TARGETARCH}" >&2
        exit 1
        ;;
esac
sccache_archive="sccache-v${SCCACHE_VERSION}-${sccache_arch}-unknown-linux-musl.tar.gz"
curl --fail --location --silent --show-error \
    --output "/tmp/${sccache_archive}" \
    "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/${sccache_archive}"
echo "${sccache_sha256}  /tmp/${sccache_archive}" | sha256sum --check --strict
tar --extract --gzip --file "/tmp/${sccache_archive}" --directory /tmp
install -m 0755 "/tmp/${sccache_archive%.tar.gz}/sccache" /usr/local/bin/sccache
rm -rf "/tmp/${sccache_archive}" "/tmp/${sccache_archive%.tar.gz}"
EOF

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
    --mount=type=secret,id=actions_results_url,env=ACTIONS_RESULTS_URL \
    --mount=type=secret,id=actions_runtime_token,env=ACTIONS_RUNTIME_TOKEN \
    <<EOF
set -e
if [ "${SCCACHE_ENABLED}" = "true" ]; then
    : "${ACTIONS_RESULTS_URL:?sccache requires the actions_results_url BuildKit secret}"
    : "${ACTIONS_RUNTIME_TOKEN:?sccache requires the actions_runtime_token BuildKit secret}"
    export RUSTC_WRAPPER=sccache
    export SCCACHE_GHA_ENABLED=on
    export SCCACHE_GHA_VERSION=bookshelf-api-docker-v1
fi
cargo build --locked --release
if [ "${SCCACHE_ENABLED}" = "true" ]; then
    sccache --show-stats
    sccache --stop-server
fi
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

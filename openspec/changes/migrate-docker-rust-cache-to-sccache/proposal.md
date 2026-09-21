## Why

The Docker image CI currently persists the entire Rust `target` cache through `buildkit-cache-dance`, making cache injection and extraction substantially slower than the cached Docker build itself. Moving compiled-object reuse to sccache's GitHub Actions backend removes that large directory transfer while retaining cross-run Rust compilation caching.

## What Changes

- Enable a pinned sccache binary only for CI Docker builds that explicitly opt in.
- Connect sccache directly to the GitHub Actions cache backend using BuildKit secret mounts for the runtime URL and token.
- Keep `/app/target` as a local BuildKit cache mount, but remove it from `actions/cache` and `buildkit-cache-dance` persistence.
- Keep the existing Cargo registry cache-dance behavior unchanged.
- Use a stable sccache cache namespace and expose hit, miss, read-error, and write-error statistics in CI.
- Preserve an unauthenticated, sccache-disabled default `docker build .` workflow.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docker-rust-build-cache`: Replace cross-run `/app/target` persistence with opt-in sccache-backed compilation caching while retaining Cargo registry and BuildKit layer caching.

## Impact

- Affected files: `Dockerfile`, `.github/workflows/ci.yml`, and `.github/workflows/deploy.yml`.
- Adds a pinned prebuilt sccache binary to the Docker build stage and GitHub Actions runtime credentials as ephemeral BuildKit secrets.
- Does not change application APIs, runtime image behavior, or the default local Docker build interface.

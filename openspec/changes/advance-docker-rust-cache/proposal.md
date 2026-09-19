## Why

The Docker build jobs restore Cargo cache mounts from an immutable GitHub Actions cache key, so successful builds cannot persist newly compiled artifacts once that key exists. As a result, ordinary source changes repeatedly restart from stale `/app/target` contents and spend unnecessary time recompiling release artifacts.

## What Changes

- Give every CI and deployment Docker build run a unique Cargo cache key.
- Restore the most recent cache that has the same Dockerfile and Cargo dependency lockfile inputs.
- Preserve cache-dance extraction on partial restores so each successful run saves its updated Cargo registry and target directories.
- Keep the BuildKit GHA layer cache configuration unchanged because it serves a separate cache layer.

## Capabilities

### New Capabilities

- `docker-rust-build-cache`: Defines how CI and deployment workflows restore, advance, and invalidate Docker BuildKit Cargo cache mounts.

### Modified Capabilities

None.

## Impact

The change affects the `actions/cache` configuration in `.github/workflows/ci.yml` and `.github/workflows/deploy.yml`. It does not change application APIs, Rust code, the Dockerfile build strategy, or the existing BuildKit `type=gha` layer cache.

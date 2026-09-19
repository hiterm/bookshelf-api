## Why

The Docker build jobs restore Cargo cache mounts from an immutable GitHub Actions cache key, so successful builds cannot persist newly compiled artifacts once that key exists. As a result, ordinary source changes repeatedly restart from stale `/app/target` contents and spend unnecessary time recompiling release artifacts.

## What Changes

- Give each commit built by the CI and deployment workflows its own Cargo cache key.
- Prefer the most recent cache with matching Dockerfile, workspace manifests, and lockfile inputs, then fall back through matching manifests to the Dockerfile lineage.
- Preserve commit-specific cache generations so cache-dance extracts partial restores and each successful commit build can save its updated Cargo registry and target directories.
- Keep the BuildKit GHA layer cache configuration unchanged because it serves a separate cache layer.

## Capabilities

### New Capabilities

- `docker-rust-build-cache`: Defines how CI and deployment workflows restore, advance, and invalidate Docker BuildKit Cargo cache mounts.

### Modified Capabilities

None.

## Impact

The change affects the `actions/cache` configuration in `.github/workflows/ci.yml` and `.github/workflows/deploy.yml`. It does not change application APIs, Rust code, the Dockerfile build strategy, or the existing BuildKit `type=gha` layer cache.

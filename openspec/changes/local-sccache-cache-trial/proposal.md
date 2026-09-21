## Why

The current Docker image CI transfers `/app/target` between runs. Its restore, extraction, and save cost can dominate a warm build. PR #359 tried sccache's GitHub Actions backend but encountered HTTP 429 writes, so a local sccache directory needs a separate trial.

## What Changes

- Replace cross-run `/app/target` persistence with local `/sccache` persistence through `buildkit-cache-dance` and `actions/cache`.
- Keep `/app/target` as a BuildKit cache mount and preserve the Cargo registry cache and BuildKit GHA layer cache.
- Enable sccache only for CI Docker builds; ordinary `docker build .` remains independent of sccache and GitHub credentials.
- Measure cache size, hit rate, errors, and cold and warm job times against main before deciding whether to adopt the change.

## Capabilities

### Modified Capabilities

- `docker-rust-build-cache`: Trial local sccache persistence in the image CI workflow.

## Impact

The trial changes `Dockerfile` and `.github/workflows/ci.yml`. Deployment will be updated only if measured CI results justify adoption.

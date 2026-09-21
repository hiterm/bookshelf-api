## Context

The existing cache-dance map transfers the Cargo registry and `/app/target`. PR #359 showed that the direct GHA sccache backend is unreliable under the cache service write rate limit.

## Goals

- Test whether local sccache reduces transfer and total CI time compared with target persistence.
- Preserve default local builds and the existing BuildKit cache mounts.
- Keep the GHA sccache backend and runtime credentials out of this design.

## Decisions

1. Install the official sccache v0.17.0 prebuilt binary in the build stage, checking the pinned SHA-256 from PR #359. Runtime stages do not copy it.
2. Add `SCCACHE_ENABLED=false`. The compile step exports `RUSTC_WRAPPER=sccache` and `SCCACHE_DIR=/sccache` only when enabled. Mount `/sccache` as a BuildKit cache beside `/app/target` and `/usr/local/cargo/registry`. Print statistics and stop the server after compilation.
3. CI opts in with a Docker build argument. Its cache-dance map and `actions/cache` paths persist the registry and `/sccache`; they exclude `/app/target`. Keep the existing `type=gha` layer cache.
4. Use a new `docker-rust-sccache` cache namespace with a commit-specific primary key and staged restore prefixes. This prevents restoring old target archives while respecting immutable cache keys.
5. Measure `du -sh`, file count, compressed Actions cache size, sccache requests, hit rate and errors, plus restore, inject, build, extraction, save, and job durations. Use miss-heavy and at least one warm run. If a compile layer is skipped, temporarily change only that layer's cache key for measurement, then remove the change.

## Adoption

Adopt only if repeated warm runs materially beat the roughly three-minute main baseline, the transferred cache is clearly smaller, hit rate is high, and errors are absent. Otherwise record the result and withdraw the implementation. Deploy workflow follows only after a positive CI decision.

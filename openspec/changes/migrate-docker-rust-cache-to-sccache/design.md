## Context

The image-building workflows currently move both `/usr/local/cargo/registry` and `/app/target` between `actions/cache` and BuildKit cache mounts. The target directory is large enough that cache-dance injection and extraction dominate a warm `Test Image Building` run. sccache supports GitHub Actions cache storage directly, using the runner-provided results URL and runtime token, and reports compiler-cache hits, misses, and storage errors.

The Dockerfile must continue to build outside GitHub Actions without credentials. Runtime credentials must remain ephemeral, while the sccache release and cache namespace must be stable and intentionally versioned.

## Goals / Non-Goals

**Goals:**

- Replace cross-run `/app/target` transfer with direct compiler-artifact reuse through sccache's GitHub Actions backend.
- Opt into sccache only from the image-building workflows and keep the default Docker build independent of GitHub Actions.
- Keep credentials out of Docker arguments, image configuration, layers, and build metadata.
- Preserve Cargo registry cache-dance and BuildKit layer caching.
- Make cache effectiveness and backend failures visible in build logs.

**Non-Goals:**

- Optimizing or replacing Cargo registry caching.
- Removing the `/app/target` BuildKit cache mount used within a builder.
- Providing a local sccache interface or backend.
- Changing application code, APIs, or runtime image contents.

## Decisions

1. Install the official prebuilt sccache release in the build stage, pinned by version and verified against a pinned upstream SHA-256 checksum. This avoids compiling sccache during every image build. The binary is not copied into runtime stages. Building sccache from source was rejected because its cost works against the optimization.

2. Add a non-secret build argument that enables sccache. The compile `RUN` conditionally exports `RUSTC_WRAPPER=sccache`, `SCCACHE_GHA_ENABLED=on`, and a stable `SCCACHE_GHA_VERSION`. With the default argument value, none of these settings apply and Cargo invokes rustc normally. A single stable cache version spans commits; it changes only when the cache must be intentionally invalidated.

3. Mount `ACTIONS_RESULTS_URL` and `ACTIONS_RUNTIME_TOKEN` as optional BuildKit secrets on the compile `RUN`. CI explicitly enables sccache and makes both mounts mandatory at runtime before compiling. The secret values are read only into that process environment and are never Docker `ARG` or persistent `ENV` values. The workflows use GitHub's documented runtime-variable export pattern before passing those values to `docker/build-push-action` as secrets.

4. Run `sccache --show-stats` after compilation in the same `RUN`, so the build log exposes cache hits, cache misses, cache read errors, and cache write errors without exporting a second control channel from the build container. Stop the server after reporting to flush pending writes. The build remains successful when the backend is unavailable, but visible error counters prevent treating such a run as effective caching.

5. Reduce the existing `actions/cache` path and cache-dance map to the Cargo registry only. Its key retains the current Dockerfile, manifest, lockfile, and commit generations. BuildKit's `type=gha` layer cache and `/app/target` cache mount remain unchanged.

6. Apply the same cache responsibility split and sccache secret handling to CI image validation and release image validation. This keeps the current capability contract consistent across both workflows.

## Risks / Trade-offs

- [GitHub cache rate limits or transient backend failures can reduce reuse] → Print sccache read/write error statistics and compare cold and warm CI runs before accepting the result.
- [Secrets could leak through shell tracing or image configuration] → Use BuildKit secret mounts, do not enable shell tracing, and inspect image history/config plus workflow logs.
- [sccache key compatibility changes can retain unusable entries] → Keep an explicit cache version that can be bumped for intentional invalidation; compiler identity remains part of sccache's own key.
- [A prebuilt binary may not cover every build architecture] → Select a pinned upstream asset/checksum for supported Docker build architectures and fail clearly for unsupported architectures.
- [Removing the target directory may make cold builds slower] → Retain the local BuildKit target mount and compare both a miss-heavy run and a subsequent warm run; roll back if hit rates or total time regress materially.

## Migration Plan

1. Commit and validate this OpenSpec change separately.
2. Add the pinned sccache installation and conditional compile path to the Dockerfile.
3. Update CI and release workflows to retain only the registry in cache-dance and pass the sccache opt-in plus runtime secrets.
4. Validate the local default build, workflow linters/security checks, Rust checks, secret absence, and sccache statistics.
5. Compare cold and warm `Test Image Building` timings with the known baseline. Revert the workflow opt-in and restore target cache-dance if sccache is ineffective or unreliable.

## Open Questions

None.

## Trial outcome (2026-09-21)

The implementation was trialed in PR #359 and then withdrawn rather than adopted:

- Without `ACTIONS_CACHE_SERVICE_V2=true`, sccache v0.17.0 selected the legacy cache API and every one of 1,118 writes failed with HTTP 404. The job still passed, demonstrating why write-error statistics are necessary.
- Enabling cache service v2 fixed the endpoint mismatch. The miss-heavy run then recorded 6 hits, 1,112 misses, 0 read errors, and 296 write errors. The write failures were HTTP 429 responses from GitHub's `CreateCacheEntry` endpoint with `retry-after: 1`.
- The v2 miss-heavy `Test Image Building` job took 6m31s, including a 5m23s Docker API build, 23s Cargo registry extraction, and 6s registry save. The immediately preceding main baseline took 3m05s, including a 6s Docker API build, 82s cache-dance extraction, and 13s cache save. These runs have different cache warmth, so the total-time difference is not a controlled performance comparison; nevertheless, 296 failed writes make the cache unreliable and preclude accepting this configuration.
- A warm compiler-cache run was not pursued after the high write-error rate met the rejection condition in the request. Local default `docker build .` reached Cargo without credentials or sccache, but the local filesystem ran out of space during release compilation, so a successful default local build was not established. The CI image build and health check did succeed.
- OpenSpec strict validation, actionlint 1.7.12, zizmor 1.23.1, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` passed. The frontend integration test ran only in CI and passed. No new API endpoint was added, so no new API E2E test was required; the existing CI API E2E test passed.

The source and workflow changes were reverted. Keep the change unimplemented until a reliable write strategy is available and both cold and warm runs demonstrate a net benefit.

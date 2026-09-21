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

## Trial measurements (2026-09-21)

The main baseline image jobs were 3m04s (run 35549692295) and 3m19s (run 35515239693). Both spent about 82–83s extracting the target cache after the build. The recent main cache archive was about 437 MB compressed.

The miss-heavy trial (run 35568769043) took 6m16s. Cargo release took 4m24s. sccache recorded 1,200 compile requests, 400 Rust misses, 0 Rust hits, and 0 read/write errors. After the build, `/sccache` was 286 MB with 1,837 files and the Cargo registry was 334 MB. The saved Actions archive was 351,394,964 bytes compressed.

The first controlled warm run (35569364188) restored that archive and changed only a temporary compile-layer cache-bust argument. Cargo release took 1m04s; all 400 Rust cacheable requests hit, with 0 misses and 0 read/write errors. `/sccache` remained 286 MB and the registry 334 MB. The image job took 2m56s.

The second controlled warm run (35569682116) also produced 400/400 Rust hits and no cache errors. Cargo release took 58 seconds, but the complete image job took 3m20s. Its archive was 351,394,210 bytes; the first warm archive was 351,377,437 bytes.

| Run | Restore | Inject | Docker API build | Cargo release | Extract | Save | Image job |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| main 35549692295 | 7s | 23s | 6s | layer skipped | 82s | 13s | 3m04s |
| main 35515239693 | 8s | 25s | 10s | layer skipped | 83s | 14s | 3m19s |
| trial cold 35568769043 | 1s | 1s | 4m54s | 4m24s | 35s | 11s | 6m16s |
| trial warm 35569364188 | 5s | 12s | 1m25s | 1m04s | 34s | 9s | 2m56s |
| trial warm 35569682116 | 5s | 19s | 1m27s | 58s | 39s | 7s | 3m20s |

The trial archive was about 20% smaller than the recent main archive (351 MB versus 437 MB), but the complete warm job did not consistently improve on main. The 58–64 seconds of warm compilation offset much of the extraction saving. The gain is too small and variable to justify maintaining an additional binary, cache mount, and cache namespace. The implementation and temporary cache bust were reverted; the deployment workflow remains unchanged. This change is intentionally left unimplemented and unarchived as a record of the trial.

The local default Docker release build was not completed because the host had about 1.1 GB free. The trial CI used the sccache-enabled path and passed. No API endpoint was added, so no additional API E2E test was needed. The existing image health check and frontend integration test passed in CI; frontend integration was not run locally.

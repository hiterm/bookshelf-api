## Context

Both Docker-building workflows use `actions/cache` to move the Cargo registry and `/app/target` cache mounts into and out of BuildKit through `buildkit-cache-dance`. Their fixed key changes only with `Dockerfile`, but GitHub caches are immutable, so a cache restored by an exact key is never replaced with the build's newer contents. The Dockerfile also uses BuildKit's cache mounts while each build action separately exports layer cache with `type=gha`.

GitHub defines `cache-hit` as true only for an exact primary-key match. A cache restored through `restore-keys` is a miss for this output, and a successful job saves the requested new primary key. `buildkit-cache-dance` runs its post-build extraction unless `skip-extraction` is true.

## Goals / Non-Goals

**Goals:**

- Restore the newest compatible Cargo cache for ordinary source changes.
- Persist updated Cargo registry and target contents after each successful commit build.
- Invalidate the restore lineage when either the Docker build definition or locked Rust dependencies change.
- Apply identical cache semantics to CI image validation and release image validation.

**Non-Goals:**

- Redesign the Dockerfile or Rust build stages.
- Replace or remove BuildKit's GHA layer cache.
- Share target caches across different Dockerfiles or Cargo lockfiles.
- Measure performance locally instead of from a GitHub-hosted workflow run.

## Decisions

Use `docker-rust-${{ hashFiles('Dockerfile', 'Cargo.lock') }}-${{ github.sha }}` as the primary key and `docker-rust-${{ hashFiles('Dockerfile', 'Cargo.lock') }}-` as the sole restore prefix in both workflows. The content hash forms the compatibility boundary, while the commit SHA makes the save key immutable and unique for ordinary runs. The restore prefix selects the most recently created cache within the same compatibility lineage.

Do not add broader fallback prefixes. In particular, a prefix that omits the combined hash could restore `/app/target` across Rust image/toolchain changes in `Dockerfile` or dependency changes in `Cargo.lock`, contrary to the intended invalidation boundary.

Keep `skip-extraction: ${{ steps.cache.outputs.cache-hit }}`. A first run for a commit has no exact SHA-suffixed key, even if `restore-keys` finds a cache. Consequently the value is false, cache-dance extracts the updated mounts after the Docker build, and `actions/cache` saves them under the new primary key. A rerun of the identical commit can hit exactly and skip unnecessary extraction because that key already contains the prior successful result.

Keep `cache-from: type=gha` and `cache-to: type=gha,mode=max`. Those settings cache BuildKit layers, whereas cache-dance persists mutable data held by Dockerfile `RUN --mount=type=cache` mounts; the mechanisms are complementary.

## Risks / Trade-offs

- [Every successful commit creates a new cache entry and consumes more cache storage] → GitHub's cache eviction policy bounds storage, and the newest compatible entry is the useful one.
- [Concurrent jobs with the same SHA can race to save the same primary key] → GitHub cache immutability safely leaves one complete successful save; subsequent runs can use it.
- [A dependency-only registry entry could be reusable across lockfile changes] → Treat the two cached paths as one unit and favor correctness/isolation of `/app/target` over a broader registry fallback.
- [PR-created caches have GitHub ref scope limitations] → Main-branch pushes continue advancing the default-branch lineage, while repeated runs within a PR can use that PR's lineage.

## Context

Both Docker-building workflows use `actions/cache` to move the Cargo registry and `/app/target` cache mounts into and out of BuildKit through `buildkit-cache-dance`. Their fixed key changes only with `Dockerfile`, but GitHub caches are immutable, so a cache restored by an exact key is never replaced with the build's newer contents. The Dockerfile also uses BuildKit's cache mounts while each build action separately exports layer cache with `type=gha`.

GitHub defines `cache-hit` as true only for an exact primary-key match. A cache restored through `restore-keys` is a miss for this output, and a successful job saves the requested new primary key. `buildkit-cache-dance` runs its post-build extraction unless `skip-extraction` is true.

## Goals / Non-Goals

**Goals:**

- Restore the newest compatible Cargo cache for ordinary source changes.
- Persist updated Cargo registry and target contents after each successful commit build.
- Prefer exact dependency-configuration caches while allowing Cargo to reuse compatible artifacts across lockfile and workspace-manifest changes.
- Invalidate the restore lineage when the Docker build definition changes.
- Apply identical cache semantics to CI image validation and release image validation.

**Non-Goals:**

- Redesign the Dockerfile or Rust build stages.
- Replace or remove BuildKit's GHA layer cache.
- Share target caches across different Dockerfiles.
- Measure performance locally instead of from a GitHub-hosted workflow run.

## Decisions

Use separate Dockerfile, workspace-manifest, and lockfile hashes followed by `github.sha` in the primary key. The commit SHA makes the save key immutable and unique for ordinary runs, allowing a partial restore to be extracted and saved as a newer generation.

Search restore prefixes from most to least specific: first the same Dockerfile, all `**/Cargo.toml` files, and Cargo.lock; then the same Dockerfile and workspace manifests across lockfile changes; finally the same Dockerfile across manifest changes. Cargo fingerprints determine which restored artifacts remain reusable. No prefix omits the Dockerfile hash, so a toolchain or Docker build-environment change starts a new hard lineage.

Keep `skip-extraction: ${{ steps.cache.outputs.cache-hit }}`. A first run for a commit has no exact SHA-suffixed key, even if `restore-keys` finds a cache. Consequently the value is false, cache-dance extracts the updated mounts after the Docker build, and `actions/cache` saves them under the new primary key. A rerun of the identical commit can hit exactly and skip unnecessary extraction because that key already contains the prior successful result.

Keep `cache-from: type=gha` and `cache-to: type=gha,mode=max`. Those settings cache BuildKit layers, whereas cache-dance persists mutable data held by Dockerfile `RUN --mount=type=cache` mounts; the mechanisms are complementary.

## Risks / Trade-offs

- [Every successful commit creates a new cache entry and consumes more cache storage] → GitHub's cache eviction policy bounds storage, and the newest compatible entry is the useful one.
- [Concurrent jobs with the same SHA can race to save the same primary key] → GitHub cache immutability safely leaves one complete successful save; subsequent runs can use it.
- [A broad fallback may download artifacts Cargo cannot reuse] → Search the exact dependency configuration first and rely on Cargo fingerprints for correctness when a broader fallback is needed.
- [PR-created caches have GitHub ref scope limitations] → Main-branch pushes continue advancing the default-branch lineage, while repeated runs within a PR can use that PR's lineage.

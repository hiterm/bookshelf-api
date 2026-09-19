## ADDED Requirements

### Requirement: Docker Cargo caches advance after successful builds
The CI and deployment Docker build workflows SHALL use a unique primary GitHub Actions cache key for each commit and SHALL prefer the most recently created cache with the same Dockerfile, workspace manifests, and lockfile when an exact key is unavailable.

#### Scenario: Ordinary source change restores and advances the cache
- **WHEN** a workflow builds a new commit whose Dockerfile and Cargo.lock match a previously cached build
- **THEN** it restores the most recent matching Cargo registry and target cache, extracts the updated cache mounts after a successful build, and saves them under the new commit-specific key

#### Scenario: Identical commit is rerun
- **WHEN** a workflow reruns a commit whose complete primary cache key already exists
- **THEN** it restores that exact cache and MAY skip post-build cache extraction because the immutable key cannot be updated

### Requirement: Docker Cargo cache restoration uses staged boundaries
The CI and deployment Docker build workflows MUST use Dockerfile content as the hard cache-lineage boundary, MUST include every workspace Cargo.toml and Cargo.lock in the most-specific restore prefix, and SHALL fall back first across lockfile changes and then across manifest changes within the same Dockerfile lineage.

#### Scenario: Docker build definition changes
- **WHEN** Dockerfile changes
- **THEN** the workflow does not restore a Cargo target cache created for the previous Dockerfile content

#### Scenario: Locked dependencies change
- **WHEN** Cargo.lock changes
- **THEN** the workflow may restore the newest cache with matching Dockerfile and workspace manifests so Cargo can reuse compatible artifacts

#### Scenario: Workspace manifest changes
- **WHEN** any workspace Cargo.toml changes
- **THEN** the workflow may restore the newest cache with matching Dockerfile so Cargo can reuse compatible artifacts

### Requirement: Cache mechanisms retain separate responsibilities
The workflows SHALL retain the BuildKit GHA layer cache alongside buildkit-cache-dance, with cache-dance persisting Dockerfile cache-mount contents and the GHA backend persisting BuildKit layers.

#### Scenario: Docker image build uses both cache mechanisms
- **WHEN** CI or deployment builds the Docker image
- **THEN** the build uses cache-dance for `/usr/local/cargo/registry` and `/app/target` and continues to configure BuildKit GHA cache import and export

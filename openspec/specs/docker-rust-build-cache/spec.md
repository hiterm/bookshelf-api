# docker-rust-build-cache Specification

## Purpose
Define how CI and deployment workflows restore, advance, and invalidate Cargo
cache mounts used by Docker BuildKit builds.

## Requirements

### Requirement: Docker Cargo caches advance after successful builds
The CI and deployment Docker build workflows SHALL use a unique primary GitHub Actions cache key for each commit and SHALL restore the most recently created cache from the same compatibility lineage when an exact key is unavailable.

#### Scenario: Ordinary source change restores and advances the cache
- **WHEN** a workflow builds a new commit whose Dockerfile and Cargo.lock match a previously cached build
- **THEN** it restores the most recent matching Cargo registry and target cache, extracts the updated cache mounts after a successful build, and saves them under the new commit-specific key

#### Scenario: Identical commit is rerun
- **WHEN** a workflow reruns a commit whose complete primary cache key already exists
- **THEN** it restores that exact cache and MAY skip post-build cache extraction because the immutable key cannot be updated

### Requirement: Docker Cargo cache compatibility is bounded by build inputs
The CI and deployment Docker build workflows MUST include both Dockerfile and Cargo.lock content in the cache compatibility boundary and MUST NOT restore Cargo target caches through a broader fallback that omits that boundary.

#### Scenario: Docker build definition changes
- **WHEN** Dockerfile changes
- **THEN** the workflow does not restore a Cargo target cache created for the previous Dockerfile content

#### Scenario: Locked dependencies change
- **WHEN** Cargo.lock changes
- **THEN** the workflow does not restore a Cargo target cache created for the previous lockfile content

### Requirement: Cache mechanisms retain separate responsibilities
The workflows SHALL retain the BuildKit GHA layer cache alongside buildkit-cache-dance, with cache-dance persisting Dockerfile cache-mount contents and the GHA backend persisting BuildKit layers.

#### Scenario: Docker image build uses both cache mechanisms
- **WHEN** CI or deployment builds the Docker image
- **THEN** the build uses cache-dance for `/usr/local/cargo/registry` and `/app/target` and continues to configure BuildKit GHA cache import and export

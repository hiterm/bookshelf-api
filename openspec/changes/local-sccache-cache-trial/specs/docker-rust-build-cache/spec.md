## MODIFIED Requirements

### Requirement: Docker Cargo caches advance after successful builds
The CI Docker build workflow SHALL use a unique primary GitHub Actions cache key per commit and SHALL restore the newest compatible Cargo registry and local sccache cache when an exact key is unavailable. An exact cache hit MAY skip extraction because Actions caches are immutable.

#### Scenario: New commit uses a compatible cache
- **WHEN** a new commit matches the Dockerfile, workspace manifests, and lockfile lineage
- **THEN** the workflow restores registry and sccache contents and saves their updated state under a commit-specific key

### Requirement: Docker Cargo cache restoration uses staged boundaries
The CI workflow MUST use Dockerfile content as its hard cache-lineage boundary, include workspace Cargo.toml files and Cargo.lock in the most-specific prefix, and fall back across lockfile then manifest changes. Its sccache namespace MUST differ from the former target-cache namespace.

#### Scenario: Previous target archive exists
- **WHEN** the new workflow seeks a cache
- **THEN** it does not restore an archive containing the former `/app/target` payload

### Requirement: Cache mechanisms retain separate responsibilities
The CI workflow SHALL retain BuildKit GHA layer caching and BuildKit cache mounts for `/app/target`, `/usr/local/cargo/registry`, and `/sccache`. Cross-run cache-dance and actions/cache SHALL persist only the registry and `/sccache`. CI SHALL explicitly opt into local sccache; the default Docker build SHALL require neither sccache nor GitHub credentials. The GHA sccache backend SHALL NOT be used.

#### Scenario: CI builds an image
- **WHEN** the CI Docker build runs with sccache enabled
- **THEN** its logs report sccache hit, miss, read-error, and write-error statistics

#### Scenario: Developer builds an image
- **WHEN** a developer runs `docker build .` without CI arguments
- **THEN** Cargo compiles without `RUSTC_WRAPPER=sccache` or GitHub credentials

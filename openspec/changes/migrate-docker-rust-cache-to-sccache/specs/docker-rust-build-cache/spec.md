## MODIFIED Requirements

### Requirement: Docker Cargo caches advance after successful builds
The CI and deployment Docker build workflows SHALL use a unique primary GitHub Actions cache key for each commit to persist the Cargo registry and SHALL prefer the most recently created registry cache with the same Dockerfile, workspace manifests, and lockfile when an exact key is unavailable. Rust compilation artifacts SHALL instead be reused across commits through sccache's GitHub Actions backend and its compiler-derived cache keys.

#### Scenario: Ordinary source change restores and advances caches
- **WHEN** a workflow builds a new commit whose Dockerfile and Cargo.lock match a previously built commit
- **THEN** it restores the most recent matching Cargo registry cache, saves the updated registry under the new commit-specific key, and allows sccache to reuse compatible Rust compilation artifacts

#### Scenario: Identical commit is rerun
- **WHEN** a workflow reruns a commit whose complete primary registry cache key already exists
- **THEN** it restores that exact registry cache, MAY skip registry extraction because the immutable key cannot be updated, and still allows sccache to read and write compiler artifacts

### Requirement: Docker Cargo cache restoration uses staged boundaries
The CI and deployment Docker build workflows MUST use Dockerfile content as the hard Cargo registry cache-lineage boundary, MUST include every workspace Cargo.toml and Cargo.lock in the most-specific restore prefix, and SHALL fall back first across lockfile changes and then across manifest changes within the same Dockerfile lineage. The workflows MUST use a stable sccache GitHub Actions cache version across commits and SHALL change that version only for intentional full invalidation.

#### Scenario: Docker build definition changes
- **WHEN** Dockerfile changes
- **THEN** the workflow does not restore a Cargo registry cache created for the previous Dockerfile content while sccache independently reuses only compiler artifacts whose own keys remain compatible

#### Scenario: Locked dependencies change
- **WHEN** Cargo.lock changes
- **THEN** the workflow may restore the newest registry cache with matching Dockerfile and workspace manifests while sccache independently reuses compatible compiler artifacts

#### Scenario: Workspace manifest changes
- **WHEN** any workspace Cargo.toml changes
- **THEN** the workflow may restore the newest registry cache with matching Dockerfile while sccache independently reuses compatible compiler artifacts

#### Scenario: Ordinary source commit changes
- **WHEN** source code changes without an intentional sccache cache-version bump
- **THEN** the workflow uses the same sccache namespace and relies on sccache's compiler keys rather than the commit SHA to select reusable artifacts

### Requirement: Cache mechanisms retain separate responsibilities
The workflows SHALL retain the BuildKit GHA layer cache alongside buildkit-cache-dance and sccache. Cache-dance SHALL persist only `/usr/local/cargo/registry`, sccache's GitHub Actions backend SHALL persist Rust compiler artifacts, and the BuildKit GHA backend SHALL persist BuildKit layers. `/app/target` SHALL remain a BuildKit cache mount but MUST NOT be copied through actions/cache or cache-dance between workflow runs.

#### Scenario: Docker image build uses complementary cache mechanisms
- **WHEN** CI or deployment builds the Docker image
- **THEN** the build uses cache-dance for `/usr/local/cargo/registry`, sccache for Rust compiler artifacts, and BuildKit GHA cache import and export for image layers

## ADDED Requirements

### Requirement: CI-only sccache activation is credential-safe
The Dockerfile MUST disable sccache by default and SHALL enable it only when a workflow explicitly opts in. GitHub Actions cache URL and runtime token values MUST enter the compile step through BuildKit secret mounts and MUST NOT be stored in Docker arguments, persistent environment variables, image layers, image configuration, or build metadata.

#### Scenario: Default local image build
- **WHEN** a developer runs `docker build .` without GitHub Actions variables, secrets, or sccache options
- **THEN** the image builds successfully with Cargo invoking rustc without sccache

#### Scenario: CI image build
- **WHEN** an image-building workflow explicitly enables sccache and provides the GitHub Actions results URL and runtime token as BuildKit secrets
- **THEN** Cargo invokes rustc through sccache with the GitHub Actions backend enabled and no credential is persisted in the resulting image or layers

#### Scenario: Enabled build lacks credentials
- **WHEN** sccache is explicitly enabled but either required GitHub Actions secret is absent
- **THEN** the compile step fails before invoking Cargo with a clear configuration error

### Requirement: sccache installation and effectiveness are auditable
The Docker build SHALL use a fixed prebuilt sccache release whose archive checksum is verified. An sccache-enabled image build SHALL report cache hits, cache misses, cache read errors, and cache write errors in its build log.

#### Scenario: sccache binary is installed
- **WHEN** the Docker build prepares its Rust build stage
- **THEN** it downloads the configured official release asset, verifies the pinned checksum, and installs that exact sccache version without compiling it from source

#### Scenario: Compilation completes with sccache enabled
- **WHEN** the Rust release build finishes
- **THEN** the Docker build log includes sccache statistics sufficient to distinguish hits, misses, read errors, and write errors

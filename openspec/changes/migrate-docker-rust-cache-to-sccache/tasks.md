## 1. Docker build integration

- [x] 1.1 Install a fixed official sccache release in the Docker build stage and verify its pinned checksum
- [x] 1.2 Add an opt-in sccache compile path with a stable GHA cache version, BuildKit secret mounts, clear missing-secret errors, and post-build statistics
- [ ] 1.3 Verify the default Docker build leaves sccache disabled and does not require GitHub Actions credentials

## 2. Workflow cache migration

- [x] 2.1 Remove `app-target` from the CI actions/cache path and cache-dance map while preserving Cargo registry caching
- [x] 2.2 Export the GitHub Actions cache runtime values and pass them as BuildKit secrets with the sccache opt-in argument in CI
- [x] 2.3 Apply the same registry-only cache-dance and secure sccache configuration to release image validation

## 3. Validation and measurement

- [x] 3.1 Run OpenSpec validation, actionlint, zizmor, and all mandatory Rust checks
- [ ] 3.2 Inspect Docker image configuration/history and build logs to verify credentials are not exposed and sccache reports hits, misses, read errors, and write errors
- [ ] 3.3 Run a miss-heavy and subsequent warm `Test Image Building` CI build and compare cache restore/inject, Docker build, sccache, extraction/save, and total durations with the baseline
- [x] 3.4 Assess whether non-API E2E coverage is needed and record the decision; leave frontend `test:integration` execution to CI

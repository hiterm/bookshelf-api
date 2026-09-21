## 1. Trial implementation

- [x] 1.1 Install checksum-verified sccache in the build stage and add an opt-in local cache compile path with statistics.
- [x] 1.2 Change CI cache-dance and actions/cache from target to sccache, retaining registry and BuildKit GHA layers.
- [ ] 1.3 Verify the default Docker build needs no sccache or GitHub credentials.

## 2. Validation and measurement

- [x] 2.1 Run strict OpenSpec validation, actionlint, zizmor, and all required Rust checks.
- [x] 2.2 Compare main and trial cache sizes, hit rates, errors, and cold/warm step and job durations.
- [x] 2.3 Decide adoption; withdraw the implementation and leave deploy unchanged because CI did not improve materially.
- [x] 2.4 Assess non-API E2E coverage and leave frontend integration to CI.

## 3. Adoption, if justified

- [ ] 3.1 Sync and archive OpenSpec, with OpenSpec and source changes in separate commits.
- [ ] 3.2 Open a PR, resolve CI and CodeRabbit review, and leave merge for explicit user permission.

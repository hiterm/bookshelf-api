## 1. Trial implementation

- [ ] 1.1 Install checksum-verified sccache in the build stage and add an opt-in local cache compile path with statistics.
- [ ] 1.2 Change CI cache-dance and actions/cache from target to sccache, retaining registry and BuildKit GHA layers.
- [ ] 1.3 Verify the default Docker build needs no sccache or GitHub credentials.

## 2. Validation and measurement

- [ ] 2.1 Run strict OpenSpec validation, actionlint, zizmor, and all required Rust checks.
- [ ] 2.2 Compare main and trial cache sizes, hit rates, errors, and cold/warm step and job durations.
- [ ] 2.3 Decide adoption; update deploy only if the trial improves CI materially.
- [ ] 2.4 Assess non-API E2E coverage and leave frontend integration to CI.

## 3. Adoption, if justified

- [ ] 3.1 Sync and archive OpenSpec, with OpenSpec and source changes in separate commits.
- [ ] 3.2 Open a PR, resolve CI and CodeRabbit review, and leave merge for explicit user permission.

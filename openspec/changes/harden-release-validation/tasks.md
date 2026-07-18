## 1. Release PR Status Coordination

- [x] 1.1 Serialize release workflow runs with the non-cancelling `release` concurrency group.
- [x] 1.2 Record all three release PR statuses as pending before dispatch and update dispatch-owned statuses to error after retry exhaustion.
- [x] 1.3 Dispatch CI and E2E validation independently with the release PR ref and head SHA.
- [x] 1.4 Pin dispatched CI and E2E checkout refs to the immutable release PR head SHA.

## 2. Release PR E2E Reporting

- [x] 2.1 Add typed workflow dispatch inputs to `e2e.yml` and check out the requested ref for both E2E jobs.
- [x] 2.2 Report API E2E and frontend integration outcomes as independent success, failure, or error statuses while preserving `Integration tests (bookshelf frontend)`.

## 3. Release Artifact Validation and Publication

- [x] 3.1 Build the tagged release image once and run API E2E against it with PostgreSQL and the API JWKS server.
- [x] 3.2 Transfer the validated image to the publication job, verify its identity, and push it to GHCR without rebuilding.
- [x] 3.3 Run Render deployment and frontend integration against the published image as independent jobs so frontend failure does not gate deployment.
- [x] 3.4 Rename the misleading Docker build step to reflect its actual responsibility.
- [x] 3.5 Preserve generated release OCI labels on the image before validation and publication.

## 4. Documentation

- [x] 4.1 Document the three release PR statuses and the API E2E and frontend integration gate policy in README.

## 5. Static and Local Validation

- [x] 5.1 Run `actionlint` and resolve workflow syntax, expression, dispatch input, and `needs` dependency findings.
- [x] 5.2 Run `zizmor .` and resolve GitHub Actions permission and secret-usage findings.
- [x] 5.3 Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` in order.
- [x] 5.4 Add a regression test for release image metadata wiring.

## 6. Live Release Verification

- [ ] 6.1 Confirm a release PR shows all three statuses moving independently from pending to terminal states.
- [ ] 6.2 Confirm API E2E failure prevents GHCR push and Render deployment.
- [ ] 6.3 Confirm frontend integration failure still allows GHCR push and Render deployment to complete.
- [ ] 6.4 Compare build logs and image digests to confirm the validated and published images are identical.

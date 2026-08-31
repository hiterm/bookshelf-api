## 1. Unify pull request workflows

- [x] 1.1 Restrict CI to pushes on `main` and pull request events, then remove dispatch inputs, checkout ref overrides, and the release PR status job.
- [x] 1.2 Restrict E2E to pushes on `main` and pull request events, then remove dispatch inputs, checkout ref overrides, and both release PR status jobs.

## 2. Simplify release coordination

- [x] 2.1 Remove release PR HEAD resolution, workflow dispatch retries, and manual commit-status reporting from the release workflow.
- [x] 2.2 Remove the release job's `actions: write` and `statuses: write` permissions while preserving tagpr, tag validation, release, and deploy behavior.

## 3. Verify the unified model

- [x] 3.1 Validate workflow syntax and assert the expected event triggers, permissions, and absence of release-specific dispatch/status configuration.
- [x] 3.2 Run the required Rust formatting, lint, and unit test checks; leave frontend integration execution to CI.

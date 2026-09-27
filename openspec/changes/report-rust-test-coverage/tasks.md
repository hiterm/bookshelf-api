## 1. Integrate coverage into CI

- [x] 1.1 Install a pinned `cargo-llvm-cov` in the Rust test job.
- [x] 1.2 Replace the existing test invocation with coverage measurement of the same package scope that emits file-level coverage and uncovered lines without a threshold.
- [x] 1.3 Publish the complete text report to both the step log and a readable GitHub Actions job summary without external uploads or artifacts.

## 2. Verify coverage reporting

- [x] 2.1 Add focused automated checks for the workflow's pinned tool, coverage flags, summary output, and absence of threshold, upload, and artifact configuration.
- [x] 2.2 Run OpenSpec validation and the repository's required formatting, lint, and locked test checks.
- [ ] 2.3 Confirm on the pull request that the coverage report renders in GitHub Actions and that all CI checks pass.

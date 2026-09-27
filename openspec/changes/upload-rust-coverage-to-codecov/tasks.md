## 1. Integrate Codecov with the existing Rust test job

- [x] 1.1 Update the existing `cargo-llvm-cov` invocation to generate `lcov.info` from the ordinary all-features locked Rust test run without adding another test execution.
- [x] 1.2 Add a pinned `codecov/codecov-action` step that uploads only `lcov.info`, uses GitHub OIDC, and fails on upload or missing-report errors.

## 2. Verify and deliver the integration

- [x] 2.1 Update the focused workflow contract test for LCOV generation, explicit Codecov configuration, no thresholds or artifacts, and no E2E coverage integration.
- [x] 2.2 Run OpenSpec validation and all mandatory formatting, lint, and locked test checks.
- [ ] 2.3 Confirm the pull-request upload and all CI checks, document manual Codecov/GitHub secret setup, and address CodeRabbit feedback through approval.

## 1. Integrate Codecov with the existing Rust test job

- [x] 1.1 Run the ordinary all-features locked Rust tests once, then generate both the existing text report and `lcov.info` from the same coverage data.
- [x] 1.2 Add a pinned `codecov/codecov-action` step that uploads only `lcov.info`, uses GitHub OIDC, and fails on upload or missing-report errors.

## 2. Verify and deliver the integration

- [x] 2.1 Remove the brittle workflow string-assertion script and rely on actionlint, zizmor, OpenSpec validation, and the actual coverage CI run.
- [x] 2.2 Run OpenSpec validation and all mandatory formatting, lint, and locked test checks.
- [x] 2.3 Confirm the pull-request upload and all CI checks, document Codecov onboarding and OIDC authentication, and address CodeRabbit feedback through approval.

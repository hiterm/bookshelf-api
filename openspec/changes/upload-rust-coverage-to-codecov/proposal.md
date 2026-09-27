## Why

Rust test coverage is currently visible only inside individual GitHub Actions runs, which makes coverage trends and pull-request changes difficult to track. Publishing the existing test run's LCOV output to Codecov adds durable coverage reporting without duplicating the Rust test suite or bringing E2E coverage into scope.

## What Changes

- Generate an LCOV report while running the existing `cargo-llvm-cov`-instrumented Rust test suite.
- Upload that report to Codecov with the official `codecov/codecov-action`.
- Keep coverage collection in the existing test job so the ordinary Rust tests still run only once.
- Add focused workflow checks for LCOV generation and Codecov upload configuration.
- Document the required Codecov project and GitHub secret setup.
- Exclude E2E test coverage collection and aggregation from this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rust-coverage-reporting`: Replace GitHub-Actions-only text reporting with LCOV generation and external Codecov upload while retaining the existing Rust test scope and failure behavior.

## Impact

- `.github/workflows/ci.yml` gains LCOV generation and a Codecov upload step in the existing Rust test job.
- `.github/scripts/test-coverage-workflow.sh` validates the revised workflow contract.
- Codecov becomes an external CI integration and requires repository onboarding plus an upload token stored as a GitHub Actions secret.
- Application code, API behavior, and E2E workflows are unaffected.

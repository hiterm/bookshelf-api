## Why

The Rust test suite currently reports only pass or failure, so contributors cannot inspect coverage gaps from a CI run. CI should expose actionable line coverage without requiring an external service or persistent report artifact.

## What Changes

- Measure Rust line coverage while running the existing CI test suite.
- Publish overall coverage, per-file coverage, and uncovered line numbers as text in the workflow log and GitHub Actions job summary.
- Keep coverage informational by omitting coverage thresholds and preserving test failures as the only test-related gate.
- Do not upload coverage to an external service or store HTML or workflow artifacts.

## Capabilities

### New Capabilities

- `rust-coverage-reporting`: Defines informational Rust coverage measurement and presentation in GitHub Actions.

### Modified Capabilities

None.

## Impact

- Affects `.github/workflows/ci.yml` and its Rust test tooling.
- Adds a pinned CI-only coverage tool installation; application dependencies and APIs are unchanged.
- Does not require E2E coverage because no endpoint or runtime behavior changes.

## Why

Actionlint and zizmor still run on every branch push, while CI and E2E now use the unified pull request and `main` push model. Aligning the static analysis workflows removes duplicate feature-branch runs and gives ordinary and tagpr release pull requests the same native Checks.

## What Changes

- Run actionlint and zizmor on the default `pull_request` activity types and on pushes to `main`.
- Stop running either workflow directly for standalone feature-branch pushes.
- Leave the path-limited Renovate configuration validation workflow unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-validation-pipeline`: Extend the unified PR and main-only push validation requirements to actionlint and zizmor.

## Impact

- Affected workflows: `.github/workflows/actionlint.yml` and `.github/workflows/zizmor.yml`.
- No workflow jobs, permissions, tools, or Renovate validation behavior change.

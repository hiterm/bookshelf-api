## Why

Release pull requests currently bypass the repository's normal pull request validation path through explicit workflow dispatches and hand-written commit statuses. This duplicates CI orchestration inside the release workflow and obscures the intended GitHub approval boundary for workflows triggered by tagpr's `GITHUB_TOKEN`.

## What Changes

- Run CI and E2E on every `pull_request` event and on pushes to `main`, while no longer running them directly for feature-branch pushes.
- Remove release-PR-only workflow dispatch inputs, checkout overrides, aggregate status jobs, and status-write permissions from CI and E2E.
- Remove release PR validation dispatching, retries, and manual commit-status reporting from the release workflow.
- Keep tagpr on `GITHUB_TOKEN` and intentionally require a human to approve the resulting release PR workflows before CI and E2E begin.
- Limit the release workflow to updating or creating the release PR and, after merge, validating the release tag and invoking the existing deploy workflow.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-validation-pipeline`: Replace release-PR-specific dispatch and status requirements with a unified pull request validation model and explicit human approval for tagpr-generated release PR workflows, while preserving release publication validation and deployment behavior.

## Impact

- Affected workflows: `.github/workflows/ci.yml`, `.github/workflows/e2e.yml`, and `.github/workflows/release.yml`.
- Affected GitHub behavior: release PR checks appear as ordinary pull request workflow runs and remain approval-required until a human selects `Approve workflows to run`.
- No PAT or GitHub App token is introduced, branch protection is not changed, and the deploy reusable workflow is unchanged.

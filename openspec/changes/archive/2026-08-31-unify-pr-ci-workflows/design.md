## Context

CI and E2E currently run on every push. A second path allows the release workflow to dispatch both workflows against a release PR SHA and then synthesize three commit statuses. That path exists because tagpr creates or updates its release PR with `GITHUB_TOKEN`, whose pull request workflow runs require explicit repository approval.

The desired model accepts that approval boundary. Normal PRs and tagpr release PRs use the same event context, jobs, and check reporting; only the release PR may wait for human approval before those jobs start.

## Goals / Non-Goals

**Goals:**

- Give ordinary and release PRs one `pull_request`-based CI and E2E path.
- Retain CI and E2E after pushes to `main` without running them for standalone feature-branch pushes.
- Remove custom dispatching and commit-status orchestration from the release workflow.
- Preserve tagpr, release tag validation, and the reusable deploy invocation unchanged apart from permissions no longer needed by the workaround.

**Non-Goals:**

- Avoiding or automating GitHub's approval requirement for tagpr-generated release PR workflows.
- Introducing a PAT or GitHub App token.
- Changing branch protection, deploy behavior, release image validation, or frontend integration behavior.

## Decisions

1. CI and E2E use `push` filtered to `main` plus an unfiltered `pull_request` trigger. GitHub's native event context selects the checked-out commit, so explicit ref inputs and checkout overrides are removed. Keeping an all-branch `push` trigger was rejected because it duplicates validation for open feature PRs.
2. Release PR workflow approval is a human action. tagpr continues to use the repository `GITHUB_TOKEN`; no privileged token or dispatch bypass is added. This retains the intended trust boundary at the cost of release PR checks not starting until approved.
3. CI and E2E rely on native workflow job conclusions. Release-specific aggregate/status jobs and `statuses: write` permissions are removed because their only consumer was the dispatch workaround.
4. The release workflow retains only tagpr coordination, release tag discovery/validation, and conditional delegation to the existing deploy reusable workflow. `actions: write` and `statuses: write` are removed from the tagpr job, while `contents: write`, `issues: read`, and `pull-requests: write` remain for tagpr and release operations.

## Risks / Trade-offs

- [A release PR can remain unvalidated while approval is pending] → Document this as intentional and require a human to select `Approve workflows to run` before merge.
- [Trigger changes could accidentally suppress validation] → Validate workflow syntax and assert that CI/E2E define both `pull_request` and `push.branches: [main]` with no `workflow_dispatch`.
- [Permission minimization could prevent tagpr operations] → Retain tagpr's existing content, issue, and pull request permissions; only permissions exclusively used by workflow dispatch and status APIs are removed.
- [Removing feature-branch push runs changes check timing] → Pull request validation remains comprehensive, and `main` continues to be validated after merge.

## Migration Plan

1. Merge the trigger and workaround removals together so the native pull request path replaces the dispatch path atomically.
2. Confirm the PR itself receives CI and E2E through `pull_request`.
3. On the next tagpr-generated release PR, approve the pending workflows manually and verify both native workflows run.

Rollback consists of reverting this workflow-only change; no persistent data or external deployment configuration is migrated.

## Open Questions

None.

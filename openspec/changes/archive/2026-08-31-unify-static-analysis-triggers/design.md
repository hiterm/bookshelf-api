## Context

CI and E2E already run for the default pull request activities and pushes to `main`. Actionlint and zizmor still run for every push, which duplicates work on feature branches and does not present those tools through the same release PR approval path.

## Goals / Non-Goals

**Goals:**

- Apply the existing PR-centered trigger model to actionlint and zizmor.
- Preserve checks on `main` and allow tagpr release PR checks after human approval.

**Non-Goals:**

- Changing either static analysis job, permission set, tool version, or command.
- Changing Renovate configuration validation.
- Expanding the default `pull_request` activity types.

## Decisions

Both workflows use the same expanded YAML trigger form as CI and E2E: `push.branches: [main]` plus `pull_request`. The compact zizmor `on: [push]` form is expanded only as needed to express the branch filter and PR trigger.

## Risks / Trade-offs

- [Static analysis no longer runs for an isolated feature-branch push] → The checks run when the branch opens or updates a pull request, and continue to run after merge on `main`.
- [Trigger syntax could diverge across workflows] → Run actionlint against the complete workflow set and compare the four trigger declarations.

## Migration Plan

Merge the two trigger changes together. Existing PR synchronization will exercise both checks; rollback is a workflow-only revert.

## Open Questions

None.

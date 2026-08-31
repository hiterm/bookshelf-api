## ADDED Requirements

### Requirement: Unified pull request validation
The repository SHALL run the same CI and E2E workflows for ordinary pull requests and tagpr release pull requests through the `pull_request` event, without release-specific workflow dispatches or hand-written commit statuses.

#### Scenario: Ordinary pull request is opened or updated
- **WHEN** an ordinary pull request emits a supported `pull_request` event
- **THEN** the CI and E2E workflows run using the native pull request event context

#### Scenario: Release pull request is approved for workflow execution
- **WHEN** a human approves workflow execution for a tagpr-generated release pull request
- **THEN** the same `pull_request` CI and E2E workflows used by ordinary pull requests begin

### Requirement: Main-only push validation
The repository SHALL run CI and E2E for pushes to `main` and SHALL NOT directly run those workflows for a standalone push to another branch.

#### Scenario: Main receives a push
- **WHEN** a commit is pushed to `main`
- **THEN** both CI and E2E workflows run

#### Scenario: Feature branch receives a push
- **WHEN** a commit is pushed to a branch other than `main` without a pull request event
- **THEN** neither CI nor E2E runs directly for that push

### Requirement: Human-approved tagpr validation
Tagpr SHALL continue to use `GITHUB_TOKEN`, and the release process SHALL accept GitHub's approval requirement for pull request workflows generated or updated by that token. The repository MUST NOT use a PAT, GitHub App token, automatic workflow dispatch, or manual commit-status bypass for release pull request validation.

#### Scenario: Tagpr creates or updates a release pull request
- **WHEN** tagpr creates or updates its release pull request with `GITHUB_TOKEN`
- **THEN** its pull request CI and E2E workflows remain approval-required until a human selects `Approve workflows to run`

### Requirement: Release workflow responsibilities
The release workflow SHALL create or update the release pull request on pushes to `main`, SHALL validate and publish a release only after the release pull request is merged and a valid release tag is resolved, and SHALL delegate deployment to the existing reusable deploy workflow only when that tag is present.

#### Scenario: Tagpr updates a release pull request
- **WHEN** a push to `main` does not produce a release tag
- **THEN** the release workflow creates or updates the release pull request without dispatching validation workflows or writing validation statuses

#### Scenario: Merged release produces a valid tag
- **WHEN** tagpr creates a release tag or an existing valid release tag points at the pushed commit
- **THEN** the release workflow validates the tag and invokes the existing deploy reusable workflow with that tag

## REMOVED Requirements

### Requirement: Immediate independent release PR statuses
**Reason**: Release pull requests now use native CI and E2E workflow checks instead of synthesized status contexts.

**Migration**: Use the job results from the `pull_request` CI and E2E workflow runs after human approval.

### Requirement: Release PR E2E dispatch
**Reason**: E2E now runs from the native `pull_request` event and no longer accepts a release PR SHA through `workflow_dispatch`.

**Migration**: Approve the tagpr-generated pull request workflows and use their native event context.

### Requirement: Status contexts are informational
**Reason**: The three release-specific status contexts are removed with the dispatch workaround.

**Migration**: No branch protection migration is required; use native workflow checks for pull request validation.

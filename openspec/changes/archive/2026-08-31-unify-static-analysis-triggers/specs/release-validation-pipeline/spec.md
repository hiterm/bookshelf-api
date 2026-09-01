## MODIFIED Requirements

### Requirement: Unified pull request validation
The repository SHALL run CI, E2E, actionlint, and zizmor for ordinary pull requests and tagpr release pull requests through the `pull_request` event, without release-specific workflow dispatches or hand-written commit statuses.

#### Scenario: Ordinary pull request is opened or updated
- **WHEN** an ordinary pull request emits a supported `pull_request` event
- **THEN** the CI, E2E, actionlint, and zizmor workflows run using the native pull request event context

#### Scenario: Release pull request is approved for workflow execution
- **WHEN** a human approves workflow execution for a tagpr-generated release pull request
- **THEN** the same `pull_request` CI, E2E, actionlint, and zizmor workflows used by ordinary pull requests begin

### Requirement: Main-only push validation
The repository SHALL run CI, E2E, actionlint, and zizmor for pushes to `main` and SHALL NOT directly run those workflows for a standalone push to another branch.

#### Scenario: Main receives a push
- **WHEN** a commit is pushed to `main`
- **THEN** the CI, E2E, actionlint, and zizmor workflows run

#### Scenario: Feature branch receives a push
- **WHEN** a commit is pushed to a branch other than `main` without a pull request event
- **THEN** CI, E2E, actionlint, and zizmor do not run directly for that push

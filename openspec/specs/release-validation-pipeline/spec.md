# release-validation-pipeline Specification

## Purpose
Define the unified pull request, release validation, and deployment pipeline.

## Requirements

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

### Requirement: API E2E gates release publication
The deployment workflow MUST run API E2E against a release Docker image with PostgreSQL and the API-side JWKS server before publishing the image.

#### Scenario: API E2E fails
- **WHEN** API E2E against the release container fails
- **THEN** the workflow does not push the image to GHCR and does not trigger Render deployment

#### Scenario: API E2E succeeds
- **WHEN** API E2E against the release container succeeds
- **THEN** the workflow makes that validated image eligible for GHCR publication

### Requirement: Published image is the validated image
The deployment workflow SHALL build the release image once and SHALL push the same image that passed API E2E without rebuilding it.

#### Scenario: Validated image is published
- **WHEN** the publication job receives the validated image
- **THEN** it verifies the image identity and pushes it without invoking another image build
- **AND** the published image retains the release OCI labels applied before validation

### Requirement: Frontend compatibility does not gate deployment
After successful image publication, the workflow SHALL run Render deployment and `Integration tests (bookshelf frontend)` as independent jobs, and a frontend integration failure SHALL fail the workflow without stopping or cancelling deployment.

#### Scenario: Frontend integration fails
- **WHEN** the published release image is incompatible with the frontend `main` branch
- **THEN** the frontend integration job and workflow fail while the Render deployment remains eligible to complete

#### Scenario: Render deployment fails
- **WHEN** Render deployment fails
- **THEN** the frontend integration job remains independently eligible to complete

### Requirement: Release runs are serialized
The release workflow SHALL use the `release` concurrency group and SHALL NOT cancel an in-progress release run when a newer run is queued.

#### Scenario: A second release run starts
- **WHEN** a release workflow is already in progress
- **THEN** the newer run waits for the active run instead of cancelling it

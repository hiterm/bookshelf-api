## ADDED Requirements

### Requirement: Immediate independent release PR statuses
The release coordinator SHALL create `release-pr-ci`, `release-pr-api-e2e`, and `release-pr-frontend-integration` commit statuses for the release pull request head SHA immediately after the pull request is updated, and each status SHALL reach a terminal state independently.

#### Scenario: Validation is dispatched
- **WHEN** tagpr updates a release pull request
- **THEN** all three contexts are recorded as `pending` before their validation workflows are dispatched

#### Scenario: Validation succeeds
- **WHEN** a dispatched validation completes successfully
- **THEN** its owning context is updated to `success` without changing the other contexts

#### Scenario: Validation fails
- **WHEN** a dispatched validation completes with a test or check failure
- **THEN** its owning context is updated to `failure` without changing the other contexts

#### Scenario: Validation cannot be dispatched
- **WHEN** the coordinator exhausts its retries while dispatching a validation workflow
- **THEN** every context owned by that dispatch is updated from `pending` to `error`

#### Scenario: Validation ends without a test result
- **WHEN** a dispatched validation is cancelled or otherwise ends without success or test failure
- **THEN** its owning context is updated to `error`

### Requirement: Release PR E2E dispatch
The E2E workflow SHALL be dispatchable against a supplied release pull request ref and SHALL report API E2E and `Integration tests (bookshelf frontend)` as separate status contexts on the supplied head SHA.

#### Scenario: E2E workflow is dispatched for a release PR
- **WHEN** the release coordinator supplies a release PR ref and head SHA
- **THEN** both E2E jobs check out that ref and independently report `release-pr-api-e2e` and `release-pr-frontend-integration`

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
- **THEN** it verifies the image identity, applies release metadata, and pushes it without invoking another image build

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

### Requirement: Status contexts are informational
The release validation change SHALL document the three status contexts but SHALL NOT modify branch protection required checks.

#### Scenario: Workflow changes are deployed
- **WHEN** the release validation workflows become active
- **THEN** repository branch protection configuration is unchanged

## Why

Release pull requests currently spend time without visible validation statuses, omit API E2E coverage from the release artifact gate, build the Docker image twice, and unnecessarily couple frontend compatibility testing to artifact publication and deployment. Release validation should provide immediate, independent feedback while ensuring that only the exact API image that passed E2E is published.

## What Changes

- Publish pending statuses immediately for release PR CI, API E2E, and frontend integration validation, then report independent terminal results.
- Make the E2E workflow dispatchable against the release PR ref.
- Build the release image once, run API E2E against that image, and push the same verified image to GHCR only after API E2E succeeds.
- Run Render deployment and frontend integration testing independently after the image is pushed.
- Keep frontend integration failure visible and workflow-failing without blocking or cancelling the Render deployment.
- Document the release status contexts and gate policy.
- Leave branch protection configuration unchanged.

## Capabilities

### New Capabilities

- `release-validation-pipeline`: Defines release PR status reporting, release artifact API E2E gating, single-image publication, and non-blocking frontend compatibility validation.

### Modified Capabilities

None.

## Impact

- Affects `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `.github/workflows/e2e.yml`, `.github/workflows/deploy.yml`, and release documentation in `README.md`.
- Uses GitHub commit statuses, reusable workflow dispatch, GHCR, and the existing Render deploy hook.
- Does not change public APIs, the GraphQL schema, the database schema, or branch protection requirements.

## Why

The frontend integration job downloads both the browser runtime it uses and an unused full Chromium bundle, while also restoring a large browser cache with limited benefit. Aligning browser provisioning with the frontend repository keeps the existing fast headless-shell runtime while simplifying CI and avoiding unnecessary downloads.

## What Changes

- Install only Playwright-managed Chromium Headless Shell for frontend integration CI while retaining system dependency installation.
- Remove the Playwright browser cache from the frontend integration job without adding a replacement cache.
- Preserve the existing browser execution mode, frontend test suite, and coverage behavior.

## Capabilities

### New Capabilities

- `frontend-integration-ci`: Defines browser provisioning and compatibility requirements for the API repository's frontend integration CI job.

### Modified Capabilities

None.

## Impact

- Affected workflow: `.github/workflows/e2e.yml`, limited to `test-integration-bookshelf`.
- Affected CI resources: Playwright browser downloads and cache usage.
- No API, application runtime, frontend configuration, or test command changes.

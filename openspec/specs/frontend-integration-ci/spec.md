# frontend-integration-ci Specification

## Purpose
Define browser provisioning and compatibility requirements for the API repository's frontend integration CI job.

## Requirements

### Requirement: Frontend integration browser provisioning
The API repository's frontend integration CI job SHALL install the Playwright-managed `chromium-headless-shell` and required system dependencies without installing the regular Chromium browser bundle. The job SHALL NOT restore or save a Playwright browser cache and SHALL NOT use a system-installed Chrome browser.

#### Scenario: Provision the headless browser runtime
- **WHEN** the frontend integration CI job prepares Playwright
- **THEN** it runs `playwright install --with-deps --only-shell chromium`
- **AND** Chrome Headless Shell is available to the frontend integration test
- **AND** FFmpeg is installed when required by Playwright
- **AND** the regular Chromium or Chrome for Testing browser bundle is not downloaded

#### Scenario: Run without a browser cache
- **WHEN** the frontend integration CI job starts on a GitHub-hosted runner
- **THEN** it does not restore or save `~/.cache/ms-playwright`
- **AND** it downloads the required headless shell for that run

### Requirement: Frontend integration test compatibility
The browser provisioning optimization SHALL preserve the frontend integration test's Playwright-managed headless-shell execution mode, test suite, coverage behavior, and existing test command.

#### Scenario: Execute the frontend integration suite
- **WHEN** browser provisioning completes successfully
- **THEN** the existing frontend integration test command runs unchanged
- **AND** the test uses Playwright-managed Chrome Headless Shell rather than system-installed Chrome
- **AND** the existing test suite and coverage behavior remain unchanged

## MODIFIED Requirements

### Requirement: CI measures Rust test coverage
The continuous integration workflow SHALL measure line coverage while running the existing Rust package test suite with all features and the locked dependency graph, and SHALL produce an LCOV report from that same test execution.

#### Scenario: Tests execute with coverage instrumentation
- **WHEN** the Rust test job runs for a pull request or a push to `main`
- **THEN** the existing package tests execute once with line coverage instrumentation
- **AND** a failing test still fails the job
- **AND** the job writes the resulting line coverage to an LCOV file

### Requirement: CI publishes Rust coverage to Codecov
The continuous integration workflow SHALL upload the generated Rust LCOV report to Codecov with the official Codecov GitHub Action.

#### Scenario: Instrumented Rust tests complete successfully
- **WHEN** the Rust tests complete and generate the LCOV report
- **THEN** the workflow uploads that explicit report file to Codecov
- **AND** the upload does not discover or merge E2E coverage data

#### Scenario: Coverage upload fails
- **WHEN** Codecov rejects the upload or the expected LCOV report is missing
- **THEN** the Rust test job fails instead of silently omitting the report

### Requirement: Coverage upload uses protected authentication
The continuous integration workflow SHALL read Codecov upload credentials from a dedicated GitHub Environment secret and MUST NOT store those credentials in the repository.

#### Scenario: CI uploads coverage
- **WHEN** the Codecov action authenticates an upload
- **THEN** the job is bound to the dedicated `codecov` GitHub Environment
- **AND** it receives the token from that Environment's `CODECOV_TOKEN` secret
- **AND** no token value is present in tracked files

## REMOVED Requirements

### Requirement: CI exposes actionable text coverage
**Reason**: Codecov becomes the durable interface for overall, per-file, and pull-request coverage reporting, so duplicating the full report in the GitHub Actions summary is no longer required.

**Migration**: Inspect coverage details and trends in the Codecov project and its pull-request report.

### Requirement: Coverage remains informational and local to GitHub Actions
**Reason**: The coverage report must now be uploaded to Codecov, superseding the previous prohibition on external reporting. Coverage remains informational because this change adds no percentage threshold.

**Migration**: Configure the repository in Codecov, create a `codecov` GitHub Environment without deployment approval rules, and add the upload token as that Environment's `CODECOV_TOKEN` secret.

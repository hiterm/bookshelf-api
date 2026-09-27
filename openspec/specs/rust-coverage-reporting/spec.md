# rust-coverage-reporting Specification

## Purpose

Define Rust coverage measurement in GitHub Actions and publication to Codecov without duplicating test execution or including E2E coverage.

## Requirements

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
The continuous integration workflow SHALL authenticate Codecov uploads with GitHub OIDC and MUST NOT require a long-lived Codecov token in repository configuration.

#### Scenario: CI uploads coverage
- **WHEN** the Codecov action authenticates an upload
- **THEN** only the Rust test job receives permission to request a GitHub OIDC token
- **AND** the Codecov action uses that OIDC identity for the upload
- **AND** no Codecov token is required in GitHub Secrets or tracked files

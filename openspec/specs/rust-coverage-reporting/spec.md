# rust-coverage-reporting Specification

## Purpose

Define informational Rust coverage measurement and presentation in GitHub Actions without external reporting or persistent artifacts.

## Requirements

### Requirement: CI measures Rust test coverage
The continuous integration workflow SHALL measure line coverage while running the existing Rust package test suite with all features and the locked dependency graph.

#### Scenario: Tests execute with coverage instrumentation
- **WHEN** the Rust test job runs for a pull request or a push to `main`
- **THEN** the existing package tests execute with line coverage instrumentation
- **AND** a failing test still fails the job

### Requirement: CI exposes actionable text coverage
The continuous integration workflow SHALL expose a text report containing overall line coverage, per-file line coverage, and uncovered line numbers in both the step log and GitHub Actions job summary.

#### Scenario: Contributor inspects a completed coverage run
- **WHEN** coverage measurement completes
- **THEN** the test step log contains the complete text report
- **AND** the test job summary contains a readable, collapsible copy of the complete text report
- **AND** the report identifies the overall total, each covered source file, and uncovered line numbers

### Requirement: Coverage remains informational and local to GitHub Actions
The continuous integration workflow MUST NOT fail based on a coverage percentage and MUST NOT publish the report to an external coverage service or persist HTML or workflow artifacts.

#### Scenario: Coverage is below any particular percentage
- **WHEN** the measured coverage has any valid percentage
- **THEN** the percentage alone does not fail the job

#### Scenario: Coverage reporting completes
- **WHEN** the text report is published to the GitHub Actions log and job summary
- **THEN** no coverage data is uploaded to an external service
- **AND** no HTML report or workflow artifact is saved

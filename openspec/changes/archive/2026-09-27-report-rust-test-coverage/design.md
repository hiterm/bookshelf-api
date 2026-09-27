## Context

The `Test Suite` job prepares PostgreSQL and runs `cargo test --all-features --locked`. It does not retain or display coverage information. The repository already installs CI-only Rust tools with the pinned `taiki-e/install-action`, and GitHub Actions provides `$GITHUB_STEP_SUMMARY` for readable run-level output.

## Goals / Non-Goals

**Goals:**

- Instrument the same Rust package tests already used by CI.
- Show overall and per-file line coverage plus uncovered line numbers in both the step log and job summary.
- Keep coverage informational while retaining ordinary test failure behavior.
- Avoid external reporting services and persistent artifacts.

**Non-Goals:**

- Enforcing a minimum coverage percentage.
- Producing HTML, LCOV, or downloadable reports.
- Measuring the separate frontend or API E2E workflows.
- Changing application behavior or dependencies.

## Decisions

1. Install `cargo-llvm-cov` 0.9.0 with the existing pinned `taiki-e/install-action`. It uses Rust's LLVM source-based instrumentation, supports the pinned stable toolchain, and produces the required report directly. `cargo-tarpaulin` was rejected because LLVM instrumentation is the more direct fit for the repository's Rust toolchain and desired textual missing-line report.
2. Replace the existing `cargo test` invocation with `cargo llvm-cov --all-features --locked --show-missing-lines`. Omitting `--workspace` preserves the current root-package scope instead of pulling the separately orchestrated API E2E crate into this job. The default textual summary provides the file table and `TOTAL` row, while `--show-missing-lines` appends compact uncovered line ranges; `--text` is deliberately omitted because it additionally renders every source line. No `--fail-under-*` option is supplied, so the percentage never determines the command result; a test failure still fails CI.
3. Capture the report with `tee` so it remains visible in the normal step log, then append it inside a fenced, collapsible section in `$GITHUB_STEP_SUMMARY`. The native report includes a `TOTAL` row, per-file rows, and an `Uncovered Lines` column, avoiding a custom parser that could drift from upstream output.
4. Keep the report as an ephemeral runner file only long enough to populate the summary. No upload action, external API, HTML generation, or artifact retention is added.

## Risks / Trade-offs

- [Coverage instrumentation increases CI duration and cache usage] -> Replace the existing test execution instead of adding a duplicate coverage job, and retain the existing Rust cache.
- [A very long report is harder to scan] -> Put the complete report in a collapsed `<details>` block while leaving GitHub's searchable step log available.
- [Tool output changes across releases] -> Pin `cargo-llvm-cov` to 0.9.0 and rely only on its documented text interface.
- [A coverage tool failure still fails the test step] -> This is intentional because CI must successfully measure coverage; only the measured percentage is non-gating.

## Migration Plan

Land the workflow change and confirm its pull request run shows the coverage table in the job summary. Rollback is a workflow-only revert to the prior `cargo test` command.

## Open Questions

None.

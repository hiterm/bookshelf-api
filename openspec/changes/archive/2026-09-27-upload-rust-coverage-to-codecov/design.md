## Context

The `test` job currently replaces `cargo test` with one `cargo llvm-cov` invocation and publishes a text report to the job summary. Coverage is not retained outside a workflow run. The workflow pins third-party actions by immutable commit SHA and includes a shell contract test for its coverage configuration.

Codecov is a new external integration. Upload authentication must not require a long-lived credential in the repository. The repository owner must onboard the repository in Codecov before the required upload can succeed.

## Goals / Non-Goals

**Goals:**

- Produce an LCOV file from the ordinary Rust test suite with `cargo-llvm-cov`.
- Upload exactly that file to Codecov from the existing `test` job.
- Keep a single execution of the Rust tests and make missing reports or upload errors visible as CI failures.
- Pin the Codecov Action to an immutable commit and verify the workflow contract locally.

**Non-Goals:**

- Collecting or merging E2E test coverage.
- Enforcing a minimum coverage percentage in this repository.
- Uploading coverage from a second job or retaining LCOV as a GitHub artifact.

## Decisions

1. **Generate LCOV directly in the existing test command.** Run `cargo llvm-cov --all-features --locked --lcov --output-path lcov.info`. This preserves the package scope, locked dependency graph, test failure behavior, and one-test-run property. A separate coverage job or a preceding `cargo test` would duplicate expensive compilation and test execution.

2. **Upload an explicit file with Codecov Action v7.1.1 pinned by commit SHA.** The upload step names `lcov.info`, disables automatic report discovery, and fails CI when uploading fails. Explicit discovery prevents unrelated or future E2E coverage files from being included accidentally. The immutable SHA follows the repository's supply-chain convention; the comment retains the human-readable release.

3. **Authenticate with GitHub OIDC.** Grant only the test job `contents: read` and `id-token: write`, then set `use_oidc: true` on the Codecov action. This avoids storing and rotating a long-lived upload token while allowing Codecov to verify the workflow identity. A repository token was tested first, but CI correctly rejected the upload when that manual secret was absent. Tokenless organization settings were also considered but weaken authentication policy for all allowed uploads.

4. **Keep focused static workflow tests.** Revise the existing shell test to require the LCOV flags, explicit Codecov inputs, and pinned action, and to reject duplicate test commands, thresholds, artifact upload, or E2E coverage inputs.

## Risks / Trade-offs

- **[The Codecov project is not enabled or does not trust the GitHub identity]** → Document repository onboarding and use `fail_ci_if_error: true` so coverage publishing cannot silently regress.
- **[Fork pull requests cannot request the upstream repository's OIDC identity]** → Codecov's fork handling may upload tokenlessly for public repositories; if the repository's policy does not allow that, fork coverage behavior requires a separate policy decision.
- **[Replacing the text report removes coverage details from the job summary]** → Codecov becomes the durable presentation layer; the CI log still records `cargo-llvm-cov` and upload outcomes.
- **[External service latency slightly increases CI duration]** → Upload in the existing job after the one test run and avoid artifacts or a separate coverage job.

## Migration Plan

1. Enable the repository in Codecov; no GitHub secret is required because uploads authenticate with OIDC.
2. Confirm a pull-request upload and the resulting Codecov report/status.
3. Roll back by reverting the workflow commit; the previous text-only coverage command can then be restored without application changes.

## Open Questions

None.

## Context

The `test` job currently replaces `cargo test` with one `cargo llvm-cov` invocation and publishes a text report to the job summary. Coverage is not retained outside a workflow run. The workflow pins third-party actions by immutable commit SHA and includes a shell contract test for its coverage configuration.

Codecov is a new external integration. Upload authentication must remain secret, and pull requests from forks cannot access ordinary GitHub Actions secrets. The repository owner must onboard the repository in Codecov and configure the upload token before the required upload can succeed.

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

3. **Authenticate with a repository upload token.** Pass `${{ secrets.CODECOV_TOKEN }}` to the action. The repository owner must enable the GitHub repository in Codecov and add that value as an Actions secret. Tokenless organization settings and OIDC were considered, but both require additional organization-level policy and OIDC would broaden workflow permissions.

4. **Keep focused static workflow tests.** Revise the existing shell test to require the LCOV flags, explicit Codecov inputs, and pinned action, and to reject duplicate test commands, thresholds, artifact upload, or E2E coverage inputs.

## Risks / Trade-offs

- **[Missing or invalid `CODECOV_TOKEN` blocks the test job]** → Document the one-time setup clearly and use `fail_ci_if_error: true` so coverage publishing cannot silently regress.
- **[Fork pull requests cannot access the repository secret]** → Codecov's fork handling may upload tokenlessly for public repositories; if the repository's Codecov policy does not allow that, the owner must decide whether to enable tokenless uploads or accept/adjust fork behavior separately.
- **[Replacing the text report removes coverage details from the job summary]** → Codecov becomes the durable presentation layer; the CI log still records `cargo-llvm-cov` and upload outcomes.
- **[External service latency slightly increases CI duration]** → Upload in the existing job after the one test run and avoid artifacts or a separate coverage job.

## Migration Plan

1. Merge the workflow and contract-test update after the repository has been enabled in Codecov and `CODECOV_TOKEN` has been configured.
2. Confirm a pull-request upload and the resulting Codecov report/status.
3. Roll back by reverting the workflow commit; the previous text-only coverage command can then be restored without application changes.

## Open Questions

None.

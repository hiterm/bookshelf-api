# Adopt a tagpr release flow

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document follows `.agent/PLANS.md` from the repository root.

## Purpose / Big Picture

Production releases currently require a GitHub Release draft and a separate version-bump pull request. After this change, tagpr maintains one Release pull request containing the next version and changelog. Merging that pull request becomes the only release operation: it creates the matching semantic-version tag and GitHub Release, validates and publishes only that tagged Docker image, and then invokes the Render deployment hook. A failed deployment can be retried from the same release workflow because it rediscovers a semantic-version tag already attached to the current `main` commit.

## Progress

- [x] Milestone 1: Add manually dispatched Release PR validation and aggregate commit status. Completed 2026-07-16.
  - [x] plan updated
- [ ] Milestone 2: Add tagpr configuration, orchestration workflow, and release-note configuration; remove Release Drafter.
  - [ ] plan updated
- [ ] Milestone 3: Deploy the exact released tag through a reusable workflow and document the new production operation.
  - [ ] plan updated
- [ ] Milestone 4: After the implementation pull request merges, verify the first `2.8.2` Release PR and its production rollout.
  - [ ] plan updated

Work began on 2026-07-16 in branch `feat-adopt-tagpr-release-flow`.

## Surprises & Discoveries

- Observation: tagpr v1.20.1 returns the complete created or updated pull request as JSON in its `pull_request` output, including `head.ref` and `head.sha`.
  Evidence: `action.yml` at tag `v1.20.1` declares `pull_request`, and `tagpr.go` serializes the GitHub pull request after refreshing it.
- Observation: The first sandboxed clippy run could not download `chacha20`; rerunning the same locked command with network access succeeded without code changes.
  Evidence: the first run reported `failed to get chacha20`; the retry reported `cargo clippy: No issues found`.

## Decision Log

- Decision: Dispatch the existing CI workflow explicitly and have an `always()` aggregation job write one `release-pr-ci` commit status.
  Rationale: `GITHUB_TOKEN` changes do not reliably trigger another workflow, while `workflow_dispatch` is explicit. Aggregating all six job results also makes failed, cancelled, and skipped jobs uniformly block the operational release gate.
  Date/Author: 2026-07-16 / Codex
- Decision: Pin `Songmu/tagpr` v1.20.1 to commit `d1b8138b7a31075141b6cd64103de9485ced7ac9`.
  Rationale: A full commit SHA makes the third-party Action immutable while retaining the reviewed v1.20.1 behavior.
  Date/Author: 2026-07-16 / Codex

## Outcomes & Retrospective

Milestone 1 is complete: the existing six CI jobs can validate an explicitly supplied Release PR ref, and an aggregate job reports the result to that pull request's exact head SHA. Implementation of release orchestration remains in progress. Production rollout remains deliberately deferred until the implementation pull request has merged and tagpr has generated the first Release PR.

## Context and Orientation

`.github/workflows/ci.yml` runs six independent validation jobs. It currently runs on pushes. The Release workflow will dispatch it against tagpr's Release PR branch and supply that branch's head commit. A seventh aggregation job writes a GitHub commit status named `release-pr-ci` to that exact head commit.

`.github/workflows/release-drafter.yml` and `.github/release-drafter.yml` implement the old release flow. The workflow creates a draft GitHub Release on each `main` push, installs `cargo-edit`, and creates a `chore/bump-version` pull request. These files will be removed. `.github/release.yml` will replace the old categorization configuration for GitHub-generated release notes.

`.tagpr` will identify `Cargo.toml`, `e2e/Cargo.toml`, and `rendercom/Dockerfile` as version files. tagpr replaces the detected current semantic version in each file, then runs `cargo check --workspace` so `Cargo.lock` records the new workspace package versions. It maps merged pull requests labeled `bump-major` and `bump-minor` to the corresponding version increments and uses patch for everything else. Existing tags omit a `v` prefix, so the empty default tag prefix is retained.

`.github/workflows/release.yml` will run on pushes to `main`. It invokes tagpr with full Git history. When tagpr creates or updates a Release PR, the workflow dispatches CI, retrying transient API failures up to five times. When tagpr creates a tag, or the workflow is rerun on a `main` commit already carrying a semantic-version tag, it exposes that tag to a deployment job.

`.github/workflows/deploy.yml` currently runs on a published GitHub Release. It builds an image, checks the frontend integration against it, pushes image metadata-derived tags to GHCR, and calls Render. It will become a `workflow_call`-only reusable workflow with a required `release_tag` input. It will check out that tag and publish only `ghcr.io/hiterm/bookshelf-api:<release_tag>`, never `latest`.

## Plan of Work

First, extend `.github/workflows/ci.yml` with required dispatch inputs for the Release PR ref and head SHA. Every checkout uses the dispatched ref when present. Add an aggregate job depending on all six existing jobs, running even when dependencies fail, and write success only if every dependency result is exactly `success`.

Second, add `.tagpr`, `.github/workflows/release.yml`, and `.github/release.yml`. The release workflow checks out full history, installs the repository's pinned Rust toolchain, runs the pinned tagpr Action, dispatches Release PR validation, and detects a release tag for both first runs and retries. Remove both Release Drafter files and pin `rendercom/Dockerfile` to version `2.8.1` as tagpr's initial replacement target.

Third, convert `.github/workflows/deploy.yml` to a callable workflow. Require `release_tag`, check out that Git tag, generate only a raw Docker tag with `latest=false`, retain the existing build, frontend integration test, GHCR push, and Render hook ordering, and update cache safety comments. Update `README.md` so production deployment means reviewing and merging the tagpr Release PR.

Finally, after this implementation reaches `main`, inspect the generated `2.8.2` Release PR. Verify synchronized versions and lockfile-only workspace changes, the `release-pr-ci` status, the `2.8.2` tag and GitHub Release after merge, the exact GHCR tag without `latest`, successful integration tests, Render deployment, and production health. Record those external results here. Do not merge the new Release PR created solely by that documentation update; leave it for the next release.

## Concrete Steps

Run all commands from `/home/hiterm/ghq/github.com/hiterm/bookshelf-api`. Before each non-documentation milestone commit, run in this exact order:

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked

Also validate workflow and package metadata:

    actionlint
    zizmor .
    cargo metadata --locked --no-deps

Inspect changes with `git --no-pager diff` and commit each milestone with its required message. Include this plan with the milestone checkbox and `plan updated` subtask checked in that same commit.

## Validation and Acceptance

Local acceptance requires all Rust checks, actionlint, zizmor, and locked Cargo metadata to succeed. The implementation pull request merging to `main` must not deploy; it must produce exactly one `2.8.2` Release PR. That pull request must change both package manifests, workspace package entries in `Cargo.lock`, and `rendercom/Dockerfile` to `2.8.2`, without changing dependency crate versions. Its head commit must receive `release-pr-ci: success` after all six CI jobs succeed.

Merging the Release PR must attach tag `2.8.2` to its merge commit and create the same-named GitHub Release. Deployment must build from that tag, push only `ghcr.io/hiterm/bookshelf-api:2.8.2`, run the existing frontend integration tests before invoking Render, and finish with a healthy production service. Rerunning failed jobs after a post-tag deployment failure must rediscover `2.8.2` at the current commit and retry deployment. A later merged pull request labeled `bump-minor` must cause tagpr to propose the next minor version.

No application or database behavior changes. Therefore no new Rust unit test or API end-to-end test is warranted. The existing unit suite remains mandatory. No new API endpoint is added, so the repository's API E2E-test requirement does not apply. The retained deployment integration test exercises the packaged release.

## Idempotence and Recovery

CI dispatch and commit-status writes are safe to retry; a later status with the same `release-pr-ci` context supersedes the earlier visible state. tagpr is designed to update its single open Release PR and recognize a merged Release PR before tagging, so rerunning it does not create duplicate releases. Tag detection must accept only exact semantic-version tags attached to the current commit. If deployment fails after tagging, rerun the failed release workflow jobs; do not create another tag.

Before the first Release PR merge, reverting the implementation pull request restores the old release path if required. After a tag and GitHub Release exist, preserve them and fix forward through the same exact-tag deployment path rather than moving or deleting release history.

## Artifacts and Notes

The tagpr Action pin was verified directly from the official repository:

    d1b8138b7a31075141b6cd64103de9485ced7ac9  refs/tags/v1.20.1

Expected aggregate status context:

    release-pr-ci: success

Milestone 1 local validation completed on 2026-07-16:

    cargo fmt --check                                      passed
    cargo clippy --all-targets --locked -- -D warnings    passed
    cargo test --locked                                    139 passed
    actionlint                                             passed
    zizmor .                                               no findings
    cargo metadata --locked --no-deps                      passed

Expected first image tag:

    ghcr.io/hiterm/bookshelf-api:2.8.2

## Interfaces and Dependencies

The CI workflow dispatch interface requires string inputs `ref` and `release_pr_head_sha`. The release workflow exposes a string job output `release_tag` and passes it to `.github/workflows/deploy.yml`. The reusable deploy workflow requires a string input named `release_tag` and inherited `GITHUB_TOKEN` plus `RENDER_DEPLOY_HOOK` secrets.

GitHub CLI `gh`, preinstalled on GitHub-hosted runners, performs workflow dispatch and commit-status API calls. `Songmu/tagpr` is fixed at v1.20.1 commit `d1b8138b7a31075141b6cd64103de9485ced7ac9`. Existing pinned checkout, Rust toolchain, Docker, cache, Node, pnpm, and Rust cache Actions remain in use.

Plan revision note (2026-07-16): Created the initial self-contained implementation plan and recorded the verified tagpr Action pin and CI aggregation design.

Plan revision note (2026-07-16): Marked milestone 1 complete and recorded its local validation results and the transient dependency-download failure.

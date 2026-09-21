## Context

The `test-integration-bookshelf` job currently restores `~/.cache/ms-playwright` and runs `playwright install --with-deps chromium`. The frontend tests use Playwright's Chromium Headless Shell, so the regular Chromium bundle is unused. A prior system Chrome experiment reduced installation time but made test execution materially slower, so the browser runtime must not change.

## Goals / Non-Goals

**Goals:**

- Download only Playwright-managed Chromium Headless Shell and any required FFmpeg artifact.
- Retain `--with-deps` system dependency handling and existing timing output.
- Remove browser cache management while preserving the current test runtime and behavior.

**Non-Goals:**

- Changing Playwright configuration, test commands, workers, retries, coverage, or frontend checkout behavior.
- Changing PostgreSQL, JWKS, API build, Rust cache, or preparation parallelism.
- Using system Chrome, a Playwright container, or a replacement cache.

## Decisions

Use `pnpm exec playwright install --with-deps --only-shell chromium` in the existing frontend preparation block. `--only-shell` keeps the Playwright-managed runtime already selected by headless tests while omitting the unused regular Chromium bundle; `--with-deps` preserves system dependency setup.

Delete the dedicated `actions/cache` step for `~/.cache/ms-playwright`. The smaller required download and cache restoration overhead do not justify a replacement browser cache.

Limit the workflow edit to the browser cache step and install argument. The existing timing instrumentation remains the source for comparing preparation performance, and CI logs confirm downloaded artifacts and runtime behavior.

## Risks / Trade-offs

- [Headless Shell is downloaded on every run] → The download is smaller than the combined browser bundles, and CI timing will verify the net effect.
- [A future frontend configuration could require full Chromium] → The integration job tracks the frontend `main` branch, so CI failures will reveal incompatibility and the provisioning command can be revisited.
- [Single-run timings vary] → Compare cache, install, preparation, test, and total job timings separately rather than relying on total duration alone.

## Migration Plan

Apply the two workflow edits, validate the YAML and OpenSpec artifacts locally, and verify the frontend integration job logs in the pull request. Rollback consists of reverting the workflow commit.

## Open Questions

None.

## Context

The release workflow updates the tagpr release pull request and dispatches CI, but the `release-pr-ci` status is absent until the dispatched workflow finishes. The E2E workflow is push-only, so release PR validation does not expose API E2E or frontend compatibility as independent statuses. During tagged deployment, the Docker image is built for frontend integration testing and then rebuilt for publication, while the frontend result blocks publication and deployment.

The release path spans three workflows: `release.yml` coordinates release PR validation and tagged releases, `e2e.yml` validates a selected source ref, and `deploy.yml` validates and publishes the release artifact.

## Goals / Non-Goals

**Goals:**

- Show three release PR validation contexts immediately and drive each to an independent terminal state.
- Gate GHCR publication and Render deployment on API E2E against the release container.
- Publish the exact image that API E2E validated without rebuilding it.
- Treat frontend `main` integration as a visible compatibility signal that can fail the workflow without stopping deployment.
- Serialize release workflow runs without cancelling an active release.

**Non-Goals:**

- Adding the status contexts to branch protection required checks.
- Changing the public API, GraphQL schema, or database schema.
- Changing the frontend repository or branch under compatibility test.
- Introducing an ExecPlan for this workflow-focused change.

## Decisions

### Report three commit statuses independently

Immediately after tagpr returns a release pull request, `release.yml` writes `pending` for `release-pr-ci`, `release-pr-api-e2e`, and `release-pr-frontend-integration` against its head SHA. CI and E2E are dispatched separately with the release PR ref and head SHA. Each dispatched workflow owns the terminal status for its context.

If a dispatch cannot be created after retries, the coordinator writes `error` only for the contexts owned by that dispatch. This distinguishes infrastructure/dispatch failure from a completed validation failure. A successful job reports `success`, a failed job reports `failure`, and cancellation or another non-test terminal result reports `error`.

Alternative considered: expose workflow check runs only. This leaves a delay before checks appear and does not provide stable, explicit status contexts immediately after the release PR is updated.

### Dispatch E2E against the release PR ref

`e2e.yml` accepts `ref` and `release_pr_head_sha` through `workflow_dispatch`. Both API E2E and `Integration tests (bookshelf frontend)` check out that ref, run independently, and use always-running reporter jobs to update their own status contexts. Existing push behavior remains intact.

Alternative considered: add both tests to `ci.yml`. Keeping E2E dispatch separate preserves job ownership and makes the two compatibility signals independently observable.

### Build, validate, transfer, and push one image

The deployment workflow builds a locally loaded release image once. It starts PostgreSQL and the API-side JWKS server, starts the release container, and runs the Rust API E2E suite against it. Only after success is the image saved as a workflow artifact. A downstream publication job loads that artifact, tags it with release metadata, and pushes it to GHCR. The image ID/digest is recorded before transfer and checked after loading so publication cannot silently substitute a rebuild.

Alternative considered: use a second cache-assisted `build-push-action` invocation. Even with cache hits, that is a second build and does not make artifact identity explicit.

### Fan out deployment and frontend compatibility after publication

Render deployment and frontend integration are separate jobs that both depend only on the successful image publication job. The frontend job pulls and runs the published release image while testing `hiterm/bookshelf@main`. It does not appear in the Render job's dependency chain. Therefore a frontend failure makes the reusable workflow and release run red but cannot prevent or cancel an already eligible Render deployment.

Alternative considered: mark frontend integration `continue-on-error`. That would preserve deployment but hide the compatibility regression from the workflow conclusion.

### Serialize release coordination

`release.yml` uses concurrency group `release` with `cancel-in-progress: false`. This prevents overlapping tagpr/release coordination while allowing the earlier release run to finish rather than being cancelled by a later push.

## Risks / Trade-offs

- [Saved Docker image artifacts can be large and slower to transfer] → Compress the image archive and retain only for the workflow's publication handoff.
- [Render deployment and frontend testing can race, so the workflow may be red after production deploys] → Document this intentional policy and keep job dependencies explicit.
- [A reporter job may itself fail to call the statuses API] → Grant `statuses: write` only to reporter/coordinator jobs and use `error` for non-test terminal results where reporting succeeds.
- [Release PR validation duplicates frontend integration at tagged release time] → Accept the cost because PR compatibility feedback and release-artifact compatibility answer different questions.
- [A pushed image could be retagged incorrectly] → Compare the loaded image ID with the recorded validated image ID before pushing and publish without invoking a build.

## Migration Plan

1. Land the workflow and README changes.
2. Observe a release PR update and confirm all three pending statuses appear and finish independently.
3. Exercise API E2E and frontend failure paths before relying on the new gate.
4. If rollback is needed, revert the workflow commit; no data or API migration is required.

## Open Questions

None.

## Context

The `test-image-building` CI job already builds the production Dockerfile,
checks its TLS behavior, starts PostgreSQL and the image, and verifies
`/health`. Building a second image solely for publication would duplicate the
most expensive work. The rolling `main` image intentionally represents the
current API `main` HEAD and is not a validated release artifact.

## Goals / Non-Goals

**Goals:**

- Reuse the existing loaded production image for publication.
- Publish on every `main` push as soon as the Docker build succeeds.
- Verify the exact registry image after it has been pushed.
- Retain PR-local image validation and TLS regression coverage.

**Non-Goals:**

- Gate publication on Test Suite, Clippy, E2E, or any sibling CI job.
- Add immutable SHA tags or clean up historical package versions.
- Change release image validation or publication guarantees.
- Roll back or delete an image after a failed post-publication health check.

## Decisions

1. Keep the existing `docker/build-push-action` invocation with `load: true`
   and tag the resulting `bookshelf-api:test` image as the GHCR `:main` image.
   This avoids a second production build and keeps PR validation unchanged.
2. Grant `packages: write` only to `test-image-building`. GHCR login, tagging,
   and pushing run only for `push` events whose ref is `refs/heads/main`.
3. Apply OCI source and revision labels during the single production build so
   a published image can be traced to the repository and commit without a
   rebuild.
4. On a `main` push, publish before health validation, remove the local tags,
   explicitly pull `ghcr.io/hiterm/bookshelf-api:main`, and run that pulled
   image. A health failure fails the job but cannot undo the completed push.
5. On pull requests, run the existing local image health check without GHCR
   login or publication. The TLS regression image remains unchanged for all
   events.

## Risks / Trade-offs

- [A broken API revision can replace the rolling image] → This is the intended
  `main` HEAD contract; the Docker build itself remains the publication gate.
- [The post-publish health check can fail after consumers see the image] → Fail
  CI visibly and retain the published artifact for diagnosis instead of
  implying rollback semantics.
- [Concurrent `main` runs can race on a mutable tag] → GitHub Actions processes
  main pushes in order normally, and consumers intentionally request the latest
  available `main` image rather than an immutable revision.

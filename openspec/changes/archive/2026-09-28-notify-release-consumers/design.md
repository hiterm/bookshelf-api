## Context

The release workflow validates an image, publishes that exact image, resolves its digest, and dispatches `api-released` to `bookshelf-api-deploy`. The frontend now consumes the same event.

## Goals / Non-Goals

**Goals:** notify both consumers only after publication, preserve deployment delivery, and narrowly scope one App token.

**Non-Goals:** pre-publication notification, release-validation changes, or a general-purpose consumer registry.

## Decisions

- Rename the job to `notify_release_consumers` to match its responsibility.
- Request a token with `contents: write` for exactly `bookshelf-api-deploy` and `bookshelf`.
- Build one JSON payload with the tag and digest, then POST it to each explicit repository.
- Retain `needs: push_to_registry` so notification follows successful publication and digest resolution.

## Risks / Trade-offs

- [One dispatch fails] → The job fails visibly; retry is safe because consumers are idempotent.
- [App is absent from the frontend] → Token generation or dispatch fails without broadening scope.

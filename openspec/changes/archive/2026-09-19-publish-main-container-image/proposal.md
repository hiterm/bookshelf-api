## Why

Frontend integration CI currently checks out and release-builds the API from
source to test compatibility with `main`. Publishing the production image for
each API `main` push lets consumers test the same revision without repeating
the Rust build.

## What Changes

- Publish the production image built by API CI as
  `ghcr.io/hiterm/bookshelf-api:main` on pushes to `main`.
- Publish immediately after the Docker build, independently of other CI jobs.
- Pull the published image and verify `/health` after publication.
- Keep pull request image validation local and preserve release publication
  semantics.

## Capabilities

### New Capabilities

- `main-container-image`: Defines the rolling API `main` image, its publication
  boundary, and its post-publication health check.

### Modified Capabilities

None.

## Impact

The change affects `.github/workflows/ci.yml` and the GHCR package. It does not
change the production Dockerfile, application API, release workflow, or deploy
workflow.

# main-container-image Specification

## Purpose
Define how API CI publishes a traceable production image for the current
`main` HEAD and validates the published registry artifact without applying
release-image guarantees.
## Requirements
### Requirement: Main pushes publish a rolling production image
The API CI workflow SHALL publish the production Docker image from every push
to `main` as `ghcr.io/hiterm/bookshelf-api:main` after that image builds
successfully, without waiting for any other CI job.

#### Scenario: Production image builds on a main push
- **WHEN** a commit is pushed to `main` and its production Docker image build succeeds
- **THEN** CI publishes that built image as `ghcr.io/hiterm/bookshelf-api:main` even if another CI job is failing or incomplete

#### Scenario: Production image build fails on a main push
- **WHEN** a commit is pushed to `main` and its production Docker image build fails
- **THEN** CI does not publish `ghcr.io/hiterm/bookshelf-api:main` for that commit

#### Scenario: Pull request image builds
- **WHEN** CI builds the production Docker image for a pull request
- **THEN** CI validates the image locally and does not publish the `:main` tag

### Requirement: Published main images identify their source revision
The rolling `main` image SHALL include OCI metadata identifying its source
repository and commit revision without requiring a second production build.

#### Scenario: Consumer inspects the rolling image
- **WHEN** a consumer inspects `ghcr.io/hiterm/bookshelf-api:main`
- **THEN** the image metadata identifies the API repository and the commit SHA used for the build

### Requirement: Main image health is checked after publication
On a push to `main`, API CI SHALL explicitly pull the published
`ghcr.io/hiterm/bookshelf-api:main` image and verify its `/health` endpoint only
after publication completes.

#### Scenario: Published image is healthy
- **WHEN** CI pushes the `:main` image, pulls it from GHCR, and `/health` succeeds
- **THEN** the image-building job succeeds

#### Scenario: Published image is unhealthy
- **WHEN** CI pushes the `:main` image and the subsequent registry-image `/health` check fails
- **THEN** the image-building job fails without deleting or rolling back the already published image

### Requirement: Release image guarantees remain independent
The rolling `main` image workflow SHALL NOT change the release workflow's
validation-before-publication contract.

#### Scenario: A release image is published
- **WHEN** the release workflow processes a release image
- **THEN** its existing validation and publication gates apply independently of the rolling `main` image

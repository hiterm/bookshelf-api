## Why

The API release pipeline currently notifies only the deployment repository, leaving the frontend to discover releases later through Renovate. Publishing one validated image should immediately notify every supported release consumer.

## What Changes

- Expand post-publish notification to `bookshelf-api-deploy` and `bookshelf`.
- Limit the GitHub App token to exactly those repositories.
- Send the same version-and-digest `api-released` payload to both.
- Preserve the existing deployment notification and post-publish timing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-validation-pipeline`: Notify deployment and frontend consumers after the validated image is published.

## Impact

- Updates `.github/workflows/deploy.yml` and release-pipeline documentation.
- Requires the existing delivery App installation on both named repositories.

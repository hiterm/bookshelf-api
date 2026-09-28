## MODIFIED Requirements

### Requirement: Frontend compatibility does not gate deployment

After successful image publication, the workflow SHALL notify both `bookshelf-api-deploy` and `bookshelf` with the image version and registry digest and run `Integration tests (bookshelf frontend)` as an independent job. A frontend integration failure SHALL fail the workflow without stopping or cancelling either release consumer notification.

#### Scenario: Publication succeeds

- **WHEN** the validated release image is published with a valid digest
- **THEN** the workflow sends the same `api-released` version-and-digest payload to `bookshelf-api-deploy` and `bookshelf`

#### Scenario: Frontend integration fails

- **WHEN** the published release image is incompatible with the frontend `main` branch
- **THEN** the frontend integration job and workflow fail while both consumer notifications remain eligible to complete

#### Scenario: Deployment repository notification fails

- **WHEN** one or more release consumer notifications fail
- **THEN** the frontend integration job remains independently eligible to complete

## ADDED Requirements

### Requirement: Release notification credentials are narrowly scoped

The workflow SHALL obtain a delivery GitHub App token restricted to `bookshelf-api-deploy` and `bookshelf`.

#### Scenario: A notification token is generated

- **WHEN** post-publication notification begins
- **THEN** the installation token names only those two repositories
- **AND** the workflow token is not used for either dispatch

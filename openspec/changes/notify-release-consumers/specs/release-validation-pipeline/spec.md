## MODIFIED Requirements

### Requirement: Successful releases publish the validated image before notifying consumers

The release workflow SHALL push the exact image that passed validation before notifying `bookshelf-api-deploy` and `bookshelf`.

#### Scenario: Validation and publication succeed

- **WHEN** the release image passes validation and is published with a valid digest
- **THEN** the workflow sends `api-released` to `bookshelf-api-deploy`
- **AND** sends the same event to `bookshelf`
- **AND** both payloads contain the release version and image digest

#### Scenario: Validation or publication fails

- **WHEN** validation or publication fails
- **THEN** neither consumer is notified

### Requirement: Release notification credentials are narrowly scoped

The workflow SHALL obtain a delivery GitHub App token restricted to `bookshelf-api-deploy` and `bookshelf`.

#### Scenario: A notification token is generated

- **WHEN** post-publication notification begins
- **THEN** the installation token names only those two repositories
- **AND** the workflow token is not used for either dispatch

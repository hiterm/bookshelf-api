# repository-contract-hygiene Specification

## Purpose
TBD - created by archiving change cleanup-unused-repository-apis. Update Purpose after archive.
## Requirements
### Requirement: Repository inputs describe active behavior
The author repository SHALL accept only inputs used to execute current author mutations. Merge metadata SHALL be recorded on its Operation.

#### Scenario: Merge author
- **WHEN** an author is merged into another author
- **THEN** the source author is deleted and the merge Operation retains the source and destination IDs

#### Scenario: Delete author
- **WHEN** an author with no associated books is deleted
- **THEN** the author deletion is recorded without an extra repository argument

### Requirement: Author name resolution uses the active bulk path
The repository SHALL expose the bulk author name resolution path used by book import and SHALL not expose an unused singular variant.

#### Scenario: Resolve names
- **WHEN** book import resolves author names
- **THEN** the bulk path returns existing author IDs and records newly created authors


## ADDED Requirements

### Requirement: Imports preserve purchase dates
The system SHALL accept an optional purchase date for every bulk-imported book
and persist it without changing import batching, atomicity, or event recording.

#### Scenario: Import mixed purchase dates
- **WHEN** one import contains books with and without purchase dates
- **THEN** each created book and initial revision retain the corresponding value

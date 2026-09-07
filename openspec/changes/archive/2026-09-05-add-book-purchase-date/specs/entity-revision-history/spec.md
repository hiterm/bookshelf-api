## ADDED Requirements

### Requirement: Book revisions snapshot purchase dates
The system SHALL include the optional purchase date in every book revision and
expose it in revision history.

#### Scenario: Query historical purchase dates
- **WHEN** a book's purchase date changes across revisions
- **THEN** each history entry returns the value captured by that revision

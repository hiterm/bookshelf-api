## ADDED Requirements

### Requirement: Book restore restores purchase date
The system SHALL restore a book's purchase date from the selected revision.

#### Scenario: Restore an earlier purchase date
- **WHEN** a client restores a revision containing an earlier purchase date
- **THEN** the current book has that earlier purchase date

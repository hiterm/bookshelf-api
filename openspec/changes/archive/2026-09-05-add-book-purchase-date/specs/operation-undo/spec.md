## ADDED Requirements

### Requirement: Book undo restores purchase date
The system SHALL include purchase date when undo reconstructs prior book state.

#### Scenario: Undo a purchase date update
- **WHEN** a client undoes an operation that changed a book's purchase date
- **THEN** the current book has the pre-operation purchase date

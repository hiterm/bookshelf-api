# book-purchase-date Specification

## Purpose
TBD - created by archiving change add-book-purchase-date. Update Purpose after archive.
## Requirements
### Requirement: Books carry an optional purchase date
The system SHALL store a book purchase date as an optional calendar date with
no time or timezone component and SHALL leave existing books unset.

#### Scenario: Create with a purchase date
- **WHEN** a client creates a book with a purchase date
- **THEN** the system stores and returns that date

#### Scenario: Create without a purchase date
- **WHEN** a client creates a book with a null purchase date
- **THEN** the system stores and returns null

### Requirement: Full updates replace purchase date
The system SHALL treat the nullable purchase date as part of the existing
full-book update contract.

#### Scenario: Replace purchase date
- **WHEN** a client updates a book with a non-null purchase date
- **THEN** the system replaces the stored purchase date without changing the creation timestamp

#### Scenario: Clear purchase date
- **WHEN** a client updates a book with a null purchase date
- **THEN** the system clears the stored purchase date

### Requirement: GraphQL uses a nullable date scalar
The system SHALL expose purchase dates on book query and mutation contracts as
the nullable GraphQL `Date` scalar.

#### Scenario: Query purchase date
- **WHEN** a client selects `purchaseDate` for a book
- **THEN** the result is a `Date` value or null


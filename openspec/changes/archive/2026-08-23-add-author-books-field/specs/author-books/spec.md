## ADDED Requirements

### Requirement: Authors expose their books
The system SHALL expose a non-null `books: [Book!]!` field on the GraphQL `Author` type using the existing GraphQL `Book` representation.

#### Scenario: Query books for an author
- **WHEN** an authenticated client selects `books` for an author with related books
- **THEN** the system returns every book related to that author

#### Scenario: Query an author without books
- **WHEN** an authenticated client selects `books` for an author with no related books
- **THEN** the system returns an empty list

#### Scenario: Book has multiple authors
- **WHEN** one book is related to multiple selected authors
- **THEN** the system includes that book in each related author's `books` result

### Requirement: Author book resolution is tenant scoped
The system MUST return only books owned by the authenticated user when resolving an author's books.

#### Scenario: Another user has a related book
- **WHEN** a relationship exists for a book owned by a different user
- **THEN** that book is absent from the authenticated user's author result

### Requirement: Multiple authors use batched book lookup
The system SHALL resolve books for multiple authors through one query-use-case batch and one repository database query per DataLoader batch.

#### Scenario: Query books for multiple authors
- **WHEN** an authenticated client selects `books` for multiple authors in one GraphQL operation
- **THEN** the system passes all selected author IDs to the batched lookup once

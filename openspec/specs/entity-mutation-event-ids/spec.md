# entity-mutation-event-ids Specification

## Purpose
TBD - created by archiving change return-entity-event-ids. Update Purpose after archive.
## Requirements
### Requirement: Create mutations expose their recorded entity event
The system SHALL return the newly recorded entity create event as a required
`eventId` on `createBook` and `createAuthor` payloads, alongside the affected
entity and the logical operation's `eventSetId`.

#### Scenario: Create a Book
- **WHEN** an authenticated client successfully creates a Book
- **THEN** `createBook.eventId` identifies that Book's newly recorded create event
- **AND** the event's event-set ID equals `createBook.eventSetId`

#### Scenario: Create an Author
- **WHEN** an authenticated client successfully creates an Author
- **THEN** `createAuthor.eventId` identifies that Author's newly recorded create event
- **AND** the event's event-set ID equals `createAuthor.eventSetId`

### Requirement: Update mutations expose their recorded entity event
The system SHALL return the newly recorded entity update event as a required
`eventId` on `updateBook` and `updateAuthor` payloads, alongside the affected
entity and the logical operation's `eventSetId`.

#### Scenario: Update a Book
- **WHEN** an authenticated client successfully updates a Book
- **THEN** `updateBook.eventId` identifies that Book's newly recorded update event
- **AND** the event's event-set ID equals `updateBook.eventSetId`

#### Scenario: Update an Author
- **WHEN** an authenticated client successfully updates an Author
- **THEN** `updateAuthor.eventId` identifies that Author's newly recorded update event
- **AND** the event's event-set ID equals `updateAuthor.eventSetId`

### Requirement: Returned entity event IDs are valid restore sources
The system SHALL accept an event ID returned by a successful Book or Author
create/update mutation wherever the corresponding restore mutation accepts a
historical entity event ID.

#### Scenario: Restore from a returned Book event ID
- **WHEN** a client passes a successful Book create/update payload's `eventId` to `restoreBook`
- **THEN** the restore operation uses that recorded Book snapshot as its source

#### Scenario: Restore from a returned Author event ID
- **WHEN** a client passes a successful Author create/update payload's `eventId` to `restoreAuthor`
- **THEN** the restore operation uses that recorded Author snapshot as its source

### Requirement: Entity event IDs remain transactionally atomic
The system MUST NOT return an entity event ID when the repository operation or
transaction that records the corresponding Book or Author event fails.

#### Scenario: Event recording fails
- **WHEN** Book or Author create/update event recording fails
- **THEN** the mutation fails without returning an `eventId`

#### Scenario: Transaction completion fails
- **WHEN** the Book or Author create/update transaction does not commit successfully
- **THEN** the mutation fails without returning an `eventId`

### Requirement: Multi-event mutation contracts remain unchanged
The system SHALL NOT add a single `eventId` to delete, restore, import, or user
registration payloads as part of this capability.

#### Scenario: Inspect out-of-scope mutation payloads
- **WHEN** a client inspects delete, restore, import, or user registration payloads
- **THEN** those payloads retain their existing event identifier contract

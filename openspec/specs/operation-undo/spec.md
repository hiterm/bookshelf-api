# operation-undo Specification

## Purpose
TBD - created by archiving change redesign-history-model. Update Purpose after archive.
## Requirements
### Requirement: Undo eligibility depends on affected entities' current after-state
The system SHALL consider an owned non-baseline Operation undoable only when every entity changed by it still matches that change's after-state, regardless of later Operations affecting unrelated entities.

#### Scenario: Later Operation changes unrelated entity
- **WHEN** every target entity still matches the target Operation's after revision but another entity changed later
- **THEN** the target Operation remains undoable

#### Scenario: Target entity changed later
- **WHEN** any target entity has a later current revision than the target Operation's after revision
- **THEN** undo is rejected

#### Scenario: Target Operation deleted an entity
- **WHEN** a target change has no after revision
- **THEN** it matches only while that entity remains absent

### Requirement: Undo creates a new inverse Operation
The system SHALL execute undo as a new `undo` Operation linked to the target and SHALL apply each target change's before-state while recording fresh Revisions and OperationChanges.

#### Scenario: Update is undone
- **WHEN** revision 4 produced from revision 3 is undone while revision 4 is current
- **THEN** revision 3 content becomes current through new revision 5 and the undo change is revision 4 to revision 5

#### Scenario: Delete is undone
- **WHEN** a deletion from revision 3 to absence is undone while the entity is absent
- **THEN** revision 3 content is restored as revision 4 and the undo change is absence to revision 4

#### Scenario: Create is undone
- **WHEN** a creation from absence to revision 1 is undone while revision 1 is current
- **THEN** the entity is deleted without a deleted Revision and the undo change is revision 1 to absence

### Requirement: Multi-entity undo is atomic
The system MUST lock and revalidate every affected entity, validate all required live references, apply every inverse, and record the undo Operation in one PostgreSQL transaction.

#### Scenario: One inverse change fails
- **WHEN** any entity in a multi-entity import or merge cannot be restored or deleted
- **THEN** current state and undo history for every affected entity roll back

#### Scenario: Book before-state references missing Author
- **WHEN** undo would restore a Book Revision whose Author no longer exists
- **THEN** the complete undo fails without partial changes

### Requirement: Undo is exposed and revalidated by the API
The system SHALL expose `undoOperation(operationId)` and MUST perform eligibility checks during execution rather than trusting previously displayed eligibility information.

#### Scenario: State changes after eligibility is displayed
- **WHEN** an affected entity changes before `undoOperation` executes
- **THEN** the mutation rejects the stale undo request without partial writes

### Requirement: The model permits undo of undo
The system SHALL record undo using the same Operation, Revision, and OperationChange model so that a future undo of that undo is representable without a dedicated redo model.

#### Scenario: Inspect an undo Operation
- **WHEN** an undo succeeds
- **THEN** its changes have ordinary before and after states and its `undo_of_operation_id` identifies the target

### Requirement: Book undo restores purchase date
The system SHALL include purchase date when undo reconstructs prior book state.

#### Scenario: Undo a purchase date update
- **WHEN** a client undoes an operation that changed a book's purchase date
- **THEN** the current book has the pre-operation purchase date


## Context

Backup export is currently a use-case-specific read model. Its domain repository
trait imports `BackupData` and `BackupScope` from `use_case`, creating an invalid
outward dependency. Backup may become a domain concept when import or restore
defines a recoverable Bookshelf state model, but that design is outside this
change.

## Goals / Non-Goals

**Goals:** eliminate all `domain -> use_case` dependencies; make the PostgreSQL
backup query implementation depend only on a narrow use-case port; make the
interactor responsible for mapping the query projection to the external DTO;
retain one repeatable-read snapshot and the existing export contract.

**Non-Goals:** define a backup domain model; implement import or restore; change
the version 1 JSON schema, SQL result set, endpoints, or transaction semantics.

## Decisions

1. `BackupQueryPort` and its projection types live together under
   `use_case::port::backup`. The projection types form the complete and narrow
   contract that PostgreSQL infrastructure may know.
2. `BackupScope` and presentation-facing `BackupData` remain under
   `use_case::dto::backup`. The infrastructure implementation does not import
   that DTO module.
3. `BackupInteractor` translates scope into the port request and maps the port
   projection into the external DTO. The port performs the complete atomic
   query because splitting the reads across independent calls would lose the
   repeatable-read guarantee.
4. The concrete implementation is named `PgBackupQuery`; "Repository" is
   avoided because this is a use-case projection rather than aggregate
   persistence.
5. Infrastructure may depend on `use_case::port` only for an explicit,
   use-case-specific I/O or query-projection contract. It may not depend on
   interactors, workflow implementations, or presentation-facing DTO modules.
6. Use-case-specific types are not moved into `domain` solely to reverse an
   import. Domain ownership will be reconsidered only if import/restore gives
   backup an actual domain model.

## Risks / Trade-offs

- **[Projection and DTO types can duplicate structure]** -> Keep mapping
  explicit but minimal; share no presentation serialization concerns with the
  infrastructure port.
- **[The query port owns an atomic database-shaped operation]** -> This is a
  deliberate narrow query-port exception required for snapshot consistency,
  not a general permission for infrastructure to depend on use-case code.

## Migration Plan

Refactor in place without data migration. Run unit, database-backed repository,
and API E2E tests to prove the response contract is unchanged. Deploy in the
same backend release planned for backup export.

# Layer Boundaries

This document defines the dependency rules for the Bookshelf API. These rules
apply to new code and to code changed during maintenance.

## Layers

The API uses four layers:

- `domain` contains entities, value objects, domain errors, and persistence
  abstractions expressed only in domain terms.
- `use_case` contains application workflows, interactors, input/output DTOs,
  and ports for use-case-specific reads or external services.
- `infrastructure` implements persistence and external-service abstractions.
  SQLx and PostgreSQL details stay here.
- `presentation` translates HTTP or GraphQL requests and responses and invokes
  use cases. It does not contain persistence logic.

## Dependency direction

Source dependencies point inward:

```text
presentation ──> use_case ──> domain
                       ^          ^
                       │          │
                 infrastructure ──┘
```

In particular:

- `domain` MUST NOT import from `use_case`, `infrastructure`, or
  `presentation`.
- `use_case` MAY import from `domain`, but MUST NOT import concrete
  infrastructure or presentation types.
- `infrastructure` MAY implement abstractions owned by `domain` or `use_case`.
- `presentation` MAY depend on use-case interfaces and DTOs, but MUST NOT call
  SQLx or concrete repositories directly.

Moving a type to a lower layer only to make an import compile is not a valid
way to satisfy these rules. The type must represent a concept owned by that
layer.

## Repositories and query ports

A domain repository represents persistence of domain concepts. Its method
signatures MUST use domain entities, value objects, domain errors, or general
language types. A domain repository MUST NOT accept or return a use-case DTO.

A read model created specifically for an API workflow, report, or export does
not automatically belong to the domain. Define such an abstraction as a port
in `use_case` when its inputs or outputs are use-case-specific. Infrastructure
may implement that port with an optimized SQL projection or a purpose-specific
read-only transaction.

Use-case DTO construction and choices such as export scope belong to the
use-case layer. Infrastructure may decide how to execute the requested query,
but it must not define the application-facing contract.

## Transaction ownership

The use-case layer owns workflow transaction boundaries when it composes
multiple domain operations or repositories. The infrastructure layer owns the
database mechanics that implement those boundaries.

A use-case query port may guarantee an atomic, consistent projection when
splitting the query into independently callable repository methods would break
that guarantee. This exception does not permit domain code to depend on
use-case DTOs.

For mutation-specific transaction invariants, see
[`event-recording.md`](event-recording.md).

## Review checklist

Before adding or changing a repository or port, verify that:

- no module under `domain` imports from an outer layer;
- domain repository signatures contain no use-case or presentation DTOs;
- use-case-specific projections are represented by use-case-owned ports;
- interactors contain application decisions and mapping rather than SQL;
- concrete database types remain under `infrastructure`;
- presentation code depends on the use-case boundary rather than concrete
  infrastructure.

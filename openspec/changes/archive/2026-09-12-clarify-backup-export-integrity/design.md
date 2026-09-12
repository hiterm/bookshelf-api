## Context

The backup query already reads current data and, for full exports, retained history in one read-only repeatable-read transaction. The canonical specification additionally requires every exported reference to be revalidated and the request to fail for dangling references, but the exporter has neither a defined integrity-checking policy nor an error contract for corrupted or legacy rows.

## Goals / Non-Goals

**Goals:**

- Define backup export as faithful serialization of tenant-owned persisted state from one consistent snapshot.
- Preserve complete current-state and retained-history export, tenant isolation, deterministic ordering, and the version 1 HTTP contract.
- Strengthen E2E evidence at the final HTTP boundary.

**Non-Goals:**

- Detect or repair corrupted, legacy, dangling, or cross-tenant database references during export.
- Define which references an integrity checker would validate or how such failures would be reported.
- Add malformed-database fixtures to infrastructure or E2E tests.

## Decisions

1. Rename the consistency requirement so it describes only the single-snapshot guarantee. Requiring internal reference resolution was rejected because it gives a read-only formatter an independent database-integrity responsibility.
2. Treat database constraints and normal write paths as the producers of valid persisted relationships. The exporter serializes the rows visible to its tenant-scoped queries without a second validation pass.
3. Keep HTTP contract coverage independent from database corruption behavior. Exact-key E2E assertions verify the public version 1 schema, while representative normal writes verify value and relationship propagation.

## Risks / Trade-offs

- [A corrupted database can produce a backup with unresolved references] → Keep corruption detection outside this exporter and design a dedicated integrity policy and error contract before promising that behavior.
- [Exact-key E2E assertions require updates for intentional format evolution] → The format is explicitly versioned, so a schema change must update the versioned contract tests deliberately.

## Context

The Author delete repository parameter is discarded, while merge details already live on `NewOperation`. The singular find-or-create method has no caller; bulk resolution is the active path. Several names and comments still describe the removed Event model.

## Goals / Non-Goals

**Goals:** Remove confirmed unused repository contract surface and make current names accurate.

**Non-Goals:** Change Operation/Revision recording, database schema, GraphQL schema, or historical files.

## Decisions

Remove `DeleteAuthorExtra` and the `extra` parameter throughout the trait, implementation, calls, and mocks. The merge operation retains its typed detail in `NewOperation`. Remove the singular find-or-create method and its dedicated row projection; keep the bulk path. Rename only local variables that incorrectly call revision numbers event IDs. Update the current restore comment to describe its actual input and result.

## Risks / Trade-offs

A downstream Rust consumer could use the public trait methods; the repository is an application crate with no identified external consumer. Compile checks and focused tests catch in-repo calls. Merge detail and result behavior remain covered by existing tests.

# Remove mutation payload compatibility aliases

This ExecPlan is a living document maintained according to `.agent/PLANS.md`. The `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` sections must remain current.

## Purpose / Big Picture

GraphQL mutations currently expose each changed entity twice: once through the canonical nested entity and again through copied fields at the payload root. After this work, clients have one stable representation, while every mutation retains its event-set identifier. The generated schema and schema tests demonstrate that canonical fields remain and compatibility aliases are absent.

## Progress

- [x] (2026-07-16) Milestone 1: Review the OpenSpec change and establish the coordinated frontend-first rollout.
  - [x] plan updated
- [x] (2026-07-16) Milestone 2: Remove aliases, add schema-focused tests, and regenerate `schema.graphql`.
  - [x] plan updated
- [ ] Milestone 3: Run schema freshness, formatting, clippy, and unit tests, then commit and publish the API PR (completed: schema freshness, formatting, clippy, and 140 unit tests pass; remaining: commit and publish).
  - [ ] plan updated
- [ ] Milestone 4: Confirm cross-repository compatibility with the migrated frontend (completed: candidate SDL accepts all migrated mutation documents; remaining: full frontend typecheck is blocked by the candidate schema's unrelated `Author.yomi` requirement).
  - [ ] plan updated

## Surprises & Discoveries

- Observation: The candidate schema makes `Author.yomi` required relative to the released frontend schema, independently of this mutation payload change.
  Evidence: Candidate-SDL GraphQL Codegen succeeds for every migrated mutation, while frontend typecheck reports only existing fixtures and mappings that omit `yomi`.
- Observation: Browser E2E cannot launch on the current host because `libnspr4.so` is absent and installing OS dependencies requires an unavailable sudo password.
  Evidence: Playwright stops in `browserType.launch` before executing application steps; API schema tests and frontend unit/type checks are unaffected.

## Decision Log

- Decision: Keep `book` and `author` as the only create/update entity representations and descriptive identifiers for delete payloads.
  Rationale: This avoids copying entity fields into payload structs and makes deleted identifier ownership explicit.
  Date/Author: 2026-07-16 / Codex
- Decision: Publish the frontend migration before the API alias removal.
  Rationale: Canonical selections work against both API versions, while old alias selections fail against the new schema.
  Date/Author: 2026-07-16 / Codex

## Outcomes & Retrospective

The payload structs and resolvers now expose one canonical entity representation, the checked-in SDL is regenerated, and a schema-focused test fixes the exact allowed field sets. Schema freshness, formatting, clippy, and all 140 unit tests pass. Publication remains in progress.

## Context and Orientation

This Rust service builds its GraphQL schema from async-graphql types and writes the checked-in result to `schema.graphql`. Mutation payload structs and resolvers are located by searching for `BookMutationPayload`, `AuthorMutationPayload`, `DeleteBookPayload`, and `DeleteAuthorPayload`. `eventSetId` identifies the transaction event set and must remain present.

## Plan of Work

Remove copied book and author fields from create/update payload structs and constructors. Remove generic `id` from delete payload structs and constructors while retaining `bookId` or `authorId`. Add a schema-focused test that proves the exact allowed field sets, regenerate the schema, and update the OpenSpec task status as milestones complete.

## Concrete Steps

From `bookshelf-api`, locate payload definitions and schema generation commands, implement the edits, and run the CI schema freshness comparison. Before every commit run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` in that order.

## Validation and Acceptance

The generated SDL must expose `book` plus `eventSetId`, `author` plus `eventSetId`, and descriptive delete IDs plus `eventSetId`; it must not expose copied entity fields or generic delete `id`. All mandatory Rust checks must pass. Existing CRUD E2E tests should be run when their database and authentication environment is available.

## Idempotence and Recovery

Schema generation, formatting, linting, and tests are repeatable. No database migration is involved. If integration validation fails, restore API compatibility aliases before deployment; do not reverse the rollout order.

## Artifacts and Notes

The client portion is tracked in `bookshelf/.agent/plans/20260716-unify-mutation-payloads.md`. The OpenSpec source is `openspec/changes/unify-mutation-payloads/`.

## Interfaces and Dependencies

No new dependencies or endpoints are permitted. `BookMutationPayload` must expose only `book` and `eventSetId`; `AuthorMutationPayload` only `author` and `eventSetId`; delete payloads expose the entity-specific ID and `eventSetId`.

Plan revision note (2026-07-16): Recorded successful schema freshness, formatting, clippy, and full unit-test validation after completing the API implementation.

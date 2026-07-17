## Context

`bookshelf-api` recently changed book and author mutations to return payload objects containing the affected entity and an `eventSetId`. Compatibility fields were then added at the payload root so the existing `bookshelf` frontend could continue selecting fields such as `createBook.id`. This produced two representations of the same value and requires payload structs to copy every entity field.

The known frontend lives in the separately versioned `bookshelf` repository. Its handwritten GraphQL documents generate TypeScript SDK types, and its unit and E2E environments reproduce the GraphQL API through MSW handlers and executable mock resolvers. The current API already exposes both the compatibility aliases and the canonical nested fields, which permits a staged migration.

## Goals / Non-Goals

**Goals:**

- Establish `book` and `author` as the only entity representations in create and update payloads.
- Establish `bookId` and `authorId` as the only deleted-entity identifiers in delete payloads.
- Preserve `eventSetId` on every mutation payload.
- Update the known frontend, generated types, and mocks without interrupting CRUD behavior.
- Define a deployment order that never pairs the alias-free API with the old frontend.

**Non-Goals:**

- Changing mutation arguments or names.
- Changing `Book`, `Author`, event history, database tables, or transaction recording.
- Renaming the existing frontend file `src/graphql/deletBook.graphql`.
- Adding new endpoints or dependencies.
- Providing a permanent compatibility or versioning layer for unknown external clients.

## Decisions

The API will retain nested entity objects rather than direct payload fields. A single `Book` or `Author` GraphQL type avoids duplicating schema fields and Rust construction logic. The alternative of removing `book` and `author` would preserve the oldest frontend query shape but would make event metadata the only reason to have a payload object and would require future entity fields to be repeated.

Delete payloads will retain descriptive identifiers rather than the generic `id` alias. A delete result cannot safely require a live entity object, and `bookId` or `authorId` communicates which identifier is returned. The alternative of retaining only `id` is shorter but ambiguous across payload types.

The frontend migration will occur before API alias removal. The current API accepts canonical nested selections, so the new frontend works against both schema versions. The inverse deployment order is unsafe because GraphQL validates selected fields before executing a resolver.

Frontend create and update operations will select entity fields through `book` or `author`. Delete operations will select `bookId` or `authorId`. Although update and delete callers currently ignore their result values, each GraphQL object selection must request at least one valid field, and selecting the canonical result keeps the documents meaningful.

The frontend's unit-test MSW handlers and E2E mock resolvers will reproduce the actual payload object structure instead of returning entity objects directly. Generated GraphQL types will remain generated artifacts rather than being manually edited.

API tests will verify both positive and negative schema behavior: canonical fields remain available and compatibility aliases are absent from the generated SDL. Existing API E2E tests already exercise nested create payloads, so no new endpoint-level E2E test is required.

## Risks / Trade-offs

- [Unknown clients still select aliases] → Treat the schema change as breaking, search known repositories, communicate it in release notes, and deploy the known frontend first.
- [Frontend schema generation downloads the released API schema] → Migrate the frontend while the released API still contains both shapes; for local alias-free validation, copy the candidate schema and invoke GraphQL Code Generator directly.
- [Mocks diverge from production payload structure] → Update both MSW handlers and executable GraphQL resolvers and run their associated tests.
- [Deployment order is reversed] → Document that the frontend is forward-compatible with the new API while the old frontend is not; rollback the API first if validation errors appear.
- [Generated files are edited inconsistently] → Regenerate `schema.graphql` from Rust and TypeScript GraphQL artifacts from their configured generators, then require clean regeneration checks.

## Migration Plan

1. Update `bookshelf` GraphQL documents, production consumers, generated TypeScript types, MSW handlers, and E2E mock resolvers to use nested entities and descriptive delete identifiers.
2. Run frontend generation, unit tests, type checking, and mutation-related E2E tests.
3. Deploy or otherwise make the frontend migration available while `bookshelf-api` still supports both payload shapes.
4. Remove compatibility fields from the Rust payload structs and resolvers, regenerate `schema.graphql`, and add schema-focused unit coverage.
5. Run all mandatory Rust formatting, linting, and test checks.
6. Deploy the alias-free API after the migrated frontend.

Rollback is asymmetric. If the frontend migration causes a problem, the previous frontend remains compatible with the still-compatible API. If the alias-free API is deployed too early, restore the previous API version or temporarily restore the aliases before investigating other changes.

## Open Questions

There are no blocking design questions. Release coordination must confirm whether any client other than `bookshelf` consumes these mutations; such a client must migrate before alias removal or receive an explicit compatibility period.

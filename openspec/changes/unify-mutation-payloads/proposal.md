## Why

GraphQL mutation payloads currently expose duplicate representations of the same entity, such as both `createBook.id` and `createBook.book.id`. These compatibility aliases were added during the recent payload-object migration and should be removed before clients become more dependent on them.

## What Changes

- **BREAKING** Remove the direct entity fields from `BookMutationPayload`, leaving `book` and `eventSetId`.
- **BREAKING** Remove the direct `id` and `name` fields from `AuthorMutationPayload`, leaving `author` and `eventSetId`.
- **BREAKING** Remove the generic `id` aliases from `DeleteBookPayload` and `DeleteAuthorPayload`, retaining `bookId`, `authorId`, and `eventSetId`.
- Update the `bookshelf` frontend to select created and updated entities through `book` or `author`, and deleted identifiers through `bookId` or `authorId`.
- Coordinate the rollout so the frontend switches to the canonical fields before the API aliases are removed.

## Capabilities

### New Capabilities

- `canonical-mutation-payloads`: Defines the canonical GraphQL response shapes for book and author create, update, and delete mutations, including coordinated client compatibility requirements.

### Modified Capabilities

None.

## Impact

The change affects the GraphQL schema and mutation payload construction in this `bookshelf-api` repository, its generated `schema.graphql`, and schema-focused tests. It also affects GraphQL documents, generated TypeScript types, production consumers, MSW handlers, executable mock resolvers, and related tests in the separately versioned `bookshelf` frontend repository.

This is a breaking API schema change for any client still selecting the compatibility aliases. The known `bookshelf` client can migrate safely against the current API because the canonical nested fields already exist. No database schema, event-recording behavior, or external dependency changes are required.

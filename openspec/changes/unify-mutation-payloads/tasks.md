## 1. Prepare coordinated changes

- [ ] 1.1 Verify both repository branches and working trees, creating a non-main frontend feature branch without disturbing unrelated work
- [ ] 1.2 Record the cross-repository implementation and deployment order in each repository's required planning artifact

## 2. Migrate the bookshelf client

- [ ] 2.1 Update create and update GraphQL documents to select entity fields under `book` or `author`
- [ ] 2.2 Update delete GraphQL documents to select `bookId` or `authorId`
- [ ] 2.3 Update production consumers to read `createBook.book.id` and `createAuthor.author.id`
- [ ] 2.4 Update MSW mutation handlers to return canonical payload object shapes
- [ ] 2.5 Update executable mock API resolvers to return canonical payload object shapes
- [ ] 2.6 Regenerate frontend GraphQL types and update affected unit and E2E expectations

## 3. Validate and commit the bookshelf client

- [ ] 3.1 Run `pnpm run generate` and confirm generated operations contain no compatibility alias selections
- [ ] 3.2 Run `pnpm run lint:fix`, `pnpm run test`, and `pnpm run typecheck`
- [ ] 3.3 Run mutation-relevant mock API and demo-mode E2E tests
- [ ] 3.4 Commit the frontend migration before removing API aliases

## 4. Simplify bookshelf-api payloads

- [ ] 4.1 Remove direct book fields from `BookMutationPayload` and construct it with only `book` and `eventSetId`
- [ ] 4.2 Remove direct author fields from `AuthorMutationPayload` and construct it with only `author` and `eventSetId`
- [ ] 4.3 Remove generic `id` aliases from delete payload structs and resolvers
- [ ] 4.4 Add schema-focused unit coverage proving canonical fields remain and compatibility aliases are absent
- [ ] 4.5 Regenerate `schema.graphql` from the Rust schema

## 5. Validate and commit bookshelf-api

- [ ] 5.1 Run the schema freshness comparison used by CI
- [ ] 5.2 Run `cargo fmt --check`
- [ ] 5.3 Run `cargo clippy --all-targets --locked -- -D warnings`
- [ ] 5.4 Run `cargo test --locked`
- [ ] 5.5 Run relevant GraphQL CRUD E2E tests when the database and authentication test environment is available
- [ ] 5.6 Commit the API alias removal after all mandatory checks pass

## 6. Verify integration and rollout

- [ ] 6.1 Generate frontend types from the alias-free candidate API schema and rerun frontend type checking
- [ ] 6.2 Run the cross-repository integration suite when its environment is available
- [ ] 6.3 Deploy or release the migrated frontend before the alias-free API
- [ ] 6.4 Confirm create, update, and delete flows after both versions are deployed and document any external-client follow-up

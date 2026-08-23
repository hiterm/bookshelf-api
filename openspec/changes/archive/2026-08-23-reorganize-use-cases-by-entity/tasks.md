## 1. User Boundaries

- [x] 1.1 Add UserQueryUseCase and UserCommandUseCase with entity-scoped method names
- [x] 1.2 Implement UserQueryInteractor and UserCommandInteractor and migrate existing user unit tests

## 2. Book Boundaries

- [x] 2.1 Add BookQueryUseCase and BookQueryInteractor with find_by_id, find_all, and find_by_author_ids, and migrate query tests
- [x] 2.2 Add BookCommandUseCase and consolidate create, update, delete, import, and restore into BookCommandInteractor
- [x] 2.3 Migrate book command normal, repository-failure, transaction, commit-failure, import, and restore unit tests

## 3. Author Boundaries

- [x] 3.1 Add AuthorQueryUseCase and AuthorQueryInteractor with find_by_id, find_all, and find_by_ids, and migrate query tests
- [x] 3.2 Add AuthorCommandUseCase and consolidate create, update, delete, merge, and restore into AuthorCommandInteractor
- [x] 3.3 Migrate author command tests while preserving merge validation, lock order, transaction, event recording, restore, and failure coverage

## 4. Event Query Boundary

- [x] 4.1 Add EventQueryUseCase and consolidate book events, author events, event-set listing, and event-set lookup into EventQueryInteractor
- [x] 4.2 Migrate event query normal and failure unit tests without moving restore behavior into EventQueryInteractor

## 5. Presentation and Composition

- [x] 5.1 Refactor GraphQL Query to depend directly on User, Book, Author, and Event Query UseCases without changing fields, arguments, or outputs
- [x] 5.2 Refactor GraphQL Mutation to depend directly on User, Book, and Author Command UseCases without changing fields, inputs, outputs, or payloads
- [x] 5.3 Refactor DataLoaders to depend on entity Query UseCases and retain existing HashMap batch results
- [x] 5.4 Update presentation mocks and unit tests for the entity-scoped UseCases while preserving behavior coverage
- [x] 5.5 Replace aggregate Query/Mutation dependency-injection aliases and construction with entity-scoped concrete Interactors

## 6. Legacy Removal

- [x] 6.1 Remove QueryUseCase, QueryInteractor, MutationUseCase, MutationInteractor, and delegation-only facade tests
- [x] 6.2 Remove operation-specific UseCase traits and Interactors after confirming no references remain

## 7. Regression Verification

- [x] 7.1 Run cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, and cargo test --locked
- [x] 7.2 Run the existing integration and E2E suites without modifying their code, fixtures, or expectations
- [x] 7.3 Verify E2E assets and the GraphQL schema have no refactor-induced diff

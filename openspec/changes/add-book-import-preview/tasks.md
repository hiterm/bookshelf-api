## 1. Transaction and Author Resolution

- [x] 1.1 Add `TransactionManager::rollback`, implement PostgreSQL rollback, and cover commit, rollback, event-set cleanup, and error propagation
- [x] 1.2 Extend bulk author find-or-create results with created status while preserving bounded SQL and update repository tests for existing, new, mixed, duplicate, tenant, and rollback cases

## 2. Shared Import Execution

- [x] 2.1 Add internal import execution result and resolved-author status types without presentation dependencies
- [x] 2.2 Extract shared validation/preparation and transactional import execution from `importBooks`, preserving normalization, batch limits, event recording, and commit behavior
- [x] 2.3 Update and extend book interactor unit/integration tests for execution results, validations, repository calls, and import regressions

## 3. Preview Use Case

- [x] 3.1 Add ID-free preview DTOs and `BookCommandUseCase::preview_import`
- [x] 3.2 Implement preview through the shared import path with explicit rollback and no commit
- [x] 3.3 Add preview unit tests for author statuses, DTO conversion, validation/execution failures, rollback success, and rollback failure

## 4. GraphQL API

- [x] 4.1 Add preview GraphQL objects and author-status enum using the existing `ImportBookInput`
- [x] 4.2 Add authenticated `previewBookImport` Mutation resolver and update generated schema artifacts if present
- [x] 4.3 Add GraphQL E2E coverage for successful fields, statuses, normalization, unchanged database state, later import, validation parity, and constraint parity where reproducible

## 5. Verification and Delivery

- [x] 5.1 Verify the shared execution path has no transaction-external side effects and update architecture documentation or comments with the rollback-preview invariant
- [x] 5.2 Run formatting, clippy, locked unit/integration tests, and the complete E2E suite; inspect the final diff and confirm OpenSpec alignment
- [ ] 5.3 Commit logical changes, push the feature branch, create the PR against `main`, inspect the PR diff, and confirm CI starts and passes

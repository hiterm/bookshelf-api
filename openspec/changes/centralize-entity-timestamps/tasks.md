## 1. Domain Timestamp Inputs

- [x] 1.1 Make Author creation accept one explicit lifecycle timestamp and cover it with unit tests
- [x] 1.2 Make Book creation use one operation timestamp for both lifecycle fields

## 2. Mutation Timestamp Ownership

- [x] 2.1 Make Book and Author create and update interactors choose operation timestamps
- [x] 2.2 Make Book and Author restore preserve creation time and set update time to the restore operation time
- [x] 2.3 Make import-created Books and Authors share one operation timestamp without changing existing Authors

## 3. Persistence and Verification

- [x] 3.1 Extend AuthorRepository lookup-or-create to persist the supplied creation timestamp on inserts
- [x] 3.2 Add or update domain, interactor, and repository tests for lifecycle timestamp behavior
- [x] 3.3 Run formatting, lint, and full locked tests and record the results in the ExecPlan

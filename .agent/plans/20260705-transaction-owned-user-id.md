# Transaction-Owned User ID

**Plan update rule**: Update this document continuously as work proceeds — each milestone commit includes current checkbox state and any discoveries.

## Goal

Make `TransactionManager::begin()` the single source of truth for the user on mutating repository operations. Mutating repositories will derive `user_id` from `PgTransaction::user_id()` instead of accepting a separate `user_id` argument.

## Milestones

- [x] M0: Create this ExecPlan.
- [x] M1: Refactor transaction and repository trait signatures.
- [x] M2: Update PostgreSQL repository implementations.
- [x] M3: Update use cases and tests to the new signatures.
- [x] M4: Update architecture documentation.
- [x] M5: Run required checks and commit.

## Surprises & Discoveries

- OpenSpec change `transaction-owned-user-id` does not exist in this checkout, so implementation proceeds from the GitHub issue text and this ExecPlan.
- Read-with-transaction repository methods still accept an explicit `user_id`, but no longer perform a transaction-user mismatch check because `PgTransaction::ensure_user()` has been removed.
- `cargo test --locked --features test-with-database` cannot run in this environment because `DATABASE_URL` is not set.

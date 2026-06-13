# Guidelines for AI Agents

## Git Commands

Always use `--no-pager` flag with git commands:

```bash
git --no-pager diff
```

## Branch Workflow

Before making any changes, verify you are not on `main`:

```bash
git branch --show-current
```

If the output is `main`, create and switch to a feature branch first:

```bash
git checkout -b <branch-name>
```

Never commit or make file changes while on the `main` branch.

## Commit Messages

- Write all commit messages in English using present tense ("Add feature", not "Added feature")
- Title: 50 characters or fewer
- Body: wrap lines at 72 characters
- Keep commits granular — commit at logical breakpoints rather than batching all changes together
- Separate renames from edits: do not combine file renaming with content changes in the same commit

## Testing

When adding or modifying features, always include tests:

- **Unit tests** — mandatory. Cover the logic being added or changed.
- **E2E tests** — mandatory when a new API endpoint is added. For other
  changes, assess whether E2E tests are needed, present your conclusion to
  the user, and let the user make the final decision.

## Code Quality

- Follow existing codebase conventions and reuse available libraries
- Never introduce code that exposes or logs secrets
- When ignoring a linter or security tool finding, always add a comment
  explaining why it is safe to ignore. Place the comment on the line immediately
  before the ignore directive.

## Event Recording

Every `create`, `update`, `delete`, and `restore` operation on any entity
must record an event inside the same database transaction. This applies to
existing entities (`Book`, `Author`) and any new entity added in the future.

The transaction boundary is owned by the use-case layer via the
`TransactionManager` domain trait: an interactor calls
`transaction_manager.begin(user_id, operation)` to open a transaction,
passes the resulting transaction by `&mut` into each repository method, and
calls `transaction_manager.commit(tx)` at the end. This lets a single
interactor compose multiple repositories (e.g. the bulk import composes
`BookRepository` and `AuthorRepository`) inside one transaction.

The only event concept that crosses into the use-case layer is the choice of
`EventSetOperation` passed to `begin`. Everything else about event recording
remains exclusively in the infrastructure layer:

- `PgTransactionManager::begin` generates the `event_set` UUID and inserts the
  single `event_set` row (the one place `event_set` rows are created).
- Each `Pg*` repository method reads `tx.event_set_id()` and inserts the
  per-event `<entity>_event` rows. Domain repository traits expose only an
  associated `Transaction` type; they carry no other event knowledge.

When adding a new entity or mutation operation:

- Drive the operation inside a single transaction opened via
  `TransactionManager::begin` with the appropriate `EventSetOperation`.
- Repository methods accept `tx: &mut Self::Transaction` and read
  `tx.event_set_id()`; they must not open their own transaction or create
  `event_set` rows.
- Create a dedicated `<entity>_event` table (and `<entity>_event_author`-style
  join tables if needed) following the `book_event` / `author_event` schema.
- The `event_set` row is inserted once in `PgTransactionManager::begin`; each
  operation inserts one row into the entity's event table.
- Add a new `EventSetOperation` variant (with its `as_str` round-trip) and the
  matching `event_set_operation` value via migration (e.g. `create_foo`,
  `update_foo`).

See `.agent/plans/20260429-add-change-history.md` for the full design and
the Decision Log for rationale, and
`.agent/plans/20260612-remove-import-books-repository.md` for the move of the
transaction boundary into the use-case layer.

## Pre-commit Checks

**MANDATORY — do not skip under any circumstances**, except:
- The user explicitly grants permission to skip
- The commit contains only documentation changes (e.g. `.md` files)

Before EVERY commit, run ALL of the following in order and ensure ALL pass.
If any command fails, fix the issue before committing. Do not commit with failures.

```bash
cargo fmt --check
cargo clippy --fix --all-targets -- -D warnings
cargo test
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting, then re-run the check.

## ExecPlans

When writing complex features or significant refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation.

Store all ExecPlan files in `.agent/plans/`. Name each file with a `yyyymmdd-` prefix (the creation date) followed by a short kebab-case description of the task (e.g. `.agent/plans/20251001-add-auth-flow.md`). Always use the creation date, even for long-running tasks.

Each milestone commit must include the ExecPlan file with that milestone's checkbox and "plan updated" sub-task checked off, and any new discoveries recorded in the Surprises & Discoveries section. Do not batch plan updates across milestones.

## File Format

Always add a trailing newline at the end of files.

## Communication

- Be direct and concise
- Think and work in English
- Use the same language as the user for confirmations and final reports

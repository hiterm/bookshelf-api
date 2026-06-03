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
Event recording belongs exclusively in the infrastructure layer (inside the
`Pg*` repository implementation) — domain traits and use-case interactors
must not be aware of it.

The **use-case layer** controls the transaction boundary (`BEGIN` / `COMMIT`)
by starting a transaction and passing `&mut PgConnection` to repository methods.
The **infrastructure layer** performs the event recording inserts on that same
connection, but does not start or commit the transaction itself.

When adding a new entity or mutation operation:

- Wrap the entire operation in a single transaction started by the use-case interactor.
- Pass `&mut PgConnection` into repository methods.
- Generate a new `event_set` UUID inside the transaction.
- Create a dedicated `<entity>_event` table (and `<entity>_event_author`-style
  join tables if needed) following the `book_event` / `author_event` schema.
- Insert one row into `event_set` and one row into the entity's event table.
- Add appropriate `event_set_operation` values (e.g. `create_foo`,
  `update_foo`) via migration.

See `.agent/plans/20260429-add-change-history.md` for the full design and
the Decision Log for rationale.

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

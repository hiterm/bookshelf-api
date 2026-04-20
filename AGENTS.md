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

## Code Quality

- Follow existing codebase conventions and reuse available libraries
- Never introduce code that exposes or logs secrets
- When ignoring a linter or security tool finding, always add a comment
  explaining why it is safe to ignore. Place the comment on the line immediately
  before the ignore directive.

## Pre-commit Checks

**MANDATORY — do not skip under any circumstances**, except:
- The user explicitly grants permission to skip
- The commit contains only documentation changes (e.g. `.md` files)

Before EVERY commit, run ALL of the following in order and ensure ALL pass.
If any command fails, fix the issue before committing. Do not commit with failures.

```bash
cargo fmt --check
cargo clippy
cargo test
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting, then re-run the check.

## ExecPlans

When writing complex features or significant refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation.

Store all ExecPlan files in `.agent/plans/`. Name each file with a `yyyymmdd-` prefix (the creation date) followed by a short kebab-case description of the task (e.g. `.agent/plans/20251001-add-auth-flow.md`). Always use the creation date, even for long-running tasks.

## File Format

Always add a trailing newline at the end of files.

## Communication

- Be direct and concise
- Think and work in English
- Use the same language as the user for confirmations and final reports

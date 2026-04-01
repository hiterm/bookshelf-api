# Guidelines for AI Agents

## Git Commands

Always use `--no-pager` flag with git commands:

```bash
git --no-pager diff
```

## Branch Workflow

Always create a working branch from `main` before making changes.
Do not work directly on the `main` branch.

## Commit Messages

- Write all commit messages in English using present tense ("Add feature", not "Added feature")
- Title: 50 characters or fewer
- Body: wrap lines at 72 characters
- Keep commits granular — commit at logical breakpoints rather than batching all changes together
- Separate renames from edits: do not combine file renaming with content changes in the same commit

## Code Quality

- Follow existing codebase conventions and reuse available libraries
- Avoid adding comments unless explicitly requested
- Never introduce code that exposes or logs secrets

## Pre-commit Checks

Before committing, run the following and ensure all pass:

```bash
cargo fmt --check
cargo clippy
cargo test
```

## ExecPlans

For complex features, create a planning document in `.agent/plans/` using the format:

```
yyyymmdd-kebab-case-description.md
```

Use the creation date even if the task spans multiple sessions.

## File Format

Always add a trailing newline at the end of files.

## Communication

- Be direct and concise
- Think and work in English
- Use the same language as the user for confirmations and final reports

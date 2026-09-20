## Context

The crate exports a top-level `types` module containing `Config` and `ErrorMessage`. Repository-wide reference checks show that neither type nor the module is used by current production code. A separate `JwtConfig` continues to use `envy` and is outside this cleanup.

## Goals / Non-Goals

**Goals:**

- Remove `src/types.rs` and the corresponding declaration from `src/lib.rs`.
- Preserve runtime behavior and API and GraphQL contracts.
- Verify the crate still formats, lints, and tests successfully.

**Non-Goals:**

- Removing `envy` or changing `JwtConfig`.
- Refactoring other type modules, including `common::types`.
- Rewriting archived OpenSpec changes, completed ExecPlans, or historical changelog entries.
- Adding or changing endpoints or tests for unchanged behavior.

## Decisions

- Delete the complete top-level module instead of only `Config`, because `ErrorMessage` is also unused and an empty module has no purpose. Keeping either unused item was rejected because it would retain dead code.
- Retain `envy`, because `JwtConfig::from_env()` currently depends on it. Removing it was rejected because it would break production code.
- Rely on existing validation rather than adding tests, because no executable behavior changes and compilation detects stale module references.

## Risks / Trade-offs

- [Risk] An overlooked consumer could depend on the exported module. → Mitigation: search all current non-historical source and run full formatting, lint, and test checks.
- [Risk] Cleanup expands into unrelated historical or dependency changes. → Mitigation: restrict source edits to `src/types.rs` and `src/lib.rs`, and preserve `envy` and historical artifacts.

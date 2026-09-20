## Why

The top-level `types` module contains only `Config` and `ErrorMessage`, neither of which is referenced by current production code. Removing this dead code reduces maintenance surface without changing runtime behavior or any public API or GraphQL contract.

## What Changes

- Remove the unused top-level `types` module and its declaration.
- Keep the `envy` dependency because `JwtConfig::from_env()` still uses it.
- Do not update archived OpenSpec changes, completed ExecPlans, historical changelog entries, or other historical artifacts.
- Preserve all runtime, API, and GraphQL behavior.

## Capabilities

### New Capabilities

- `source-module-hygiene`: Defines the source-layout constraint for removing the unused top-level `types` module while preserving observable behavior.

### Modified Capabilities

None. No behavioral requirements or contracts change.

## Impact

Only `src/types.rs` and its module declaration in `src/lib.rs` are affected. Dependencies, runtime behavior, API endpoints, and GraphQL contracts remain unchanged.

## ADDED Requirements

### Requirement: Remove the unused top-level types module
The crate source SHALL omit the unused top-level `types` module and its `Config` and `ErrorMessage` definitions while preserving existing runtime, API, and GraphQL behavior.

#### Scenario: Build without the unused module
- **WHEN** the crate is built after the cleanup
- **THEN** it compiles without `src/types.rs` or a top-level `types` module declaration

#### Scenario: Preserve active environment configuration
- **WHEN** JWT configuration is loaded from environment variables
- **THEN** `JwtConfig::from_env()` continues to use the retained `envy` dependency

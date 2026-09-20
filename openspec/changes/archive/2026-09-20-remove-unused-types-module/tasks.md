## 1. Confirm Scope

- [x] 1.1 Confirm the top-level module, `Config`, and `ErrorMessage` have no current production references
- [x] 1.2 Confirm `envy` remains in active use by `JwtConfig::from_env()`

## 2. Remove Dead Code

- [x] 2.1 Delete `src/types.rs`
- [x] 2.2 Remove `pub mod types;` from `src/lib.rs`

## 3. Validate

- [x] 3.1 Run `cargo fmt --check`
- [x] 3.2 Run `cargo clippy --all-targets --locked -- -D warnings`
- [x] 3.3 Run `cargo test --locked`
- [x] 3.4 Verify the final diff is limited to the requested cleanup and OpenSpec artifacts

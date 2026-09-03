## 1. Error Classification and Disclosure

- [x] 1.1 Refactor Claims extractor errors to retain internal JWKS causes while rendering fixed safe `503` responses and sanitized `401` responses
- [x] 1.2 Add one structured server-side log for JWKS infrastructure failures without tokens or authorization headers

## 2. JWKS Fetching and Validation

- [x] 2.1 Reject JWKS HTTP 4xx/5xx responses before body decoding and preserve stage-specific internal context
- [x] 2.2 Preserve URL safety, cache reuse, concurrent miss deduplication, and one-refresh unknown-`kid` behavior

## 3. Authentication Boundary Tests

- [x] 3.1 Add tests for missing authorization, malformed JWT, missing `kid`, unsupported algorithms, and invalid signature, expiry, audience, and issuer
- [x] 3.2 Add JWKS failure tests for HTTP 500, invalid JSON, and a stable local connection failure where practical, including public-response leak assertions
- [x] 3.3 Add request-counting tests for cache reuse, successful key rotation refresh, and bounded refresh when the key remains unknown
- [x] 3.4 Confirm valid JWT GraphQL access and existing loopback/HTTPS JWKS URL behavior remain intact

## 4. Validation

- [x] 4.1 Run `cargo fmt --check`
- [x] 4.2 Run `cargo clippy --all-targets --locked -- -D warnings`
- [x] 4.3 Run `cargo test --locked`, including authentication end-to-end coverage

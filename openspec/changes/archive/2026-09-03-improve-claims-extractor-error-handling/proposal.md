## Why

The Claims extractor currently treats some JWKS endpoint failures as JSON decoding errors and exposes internal failure details to API clients. Authentication failures must remain distinguishable from temporary authentication infrastructure failures without leaking implementation details.

## What Changes

- Classify malformed or unverifiable bearer tokens, missing or unknown key IDs, and unsupported signing algorithms as `401 Unauthorized` authentication failures.
- Classify JWKS request, HTTP status, response decoding, HTTP client construction, and server-side URL configuration failures as `503 Service Unavailable`.
- Return a fixed public response for JWKS failures while retaining and logging the internal cause.
- Preserve the existing JWKS cache, concurrent fetch deduplication, and single-refresh key rotation behavior.
- Add boundary tests for Claims extraction, JWKS failures, token validation, cache use, and key rotation.

## Capabilities

### New Capabilities

- `claims-authentication`: Defines Claims extractor authentication failures, JWKS infrastructure failure responses, safe error disclosure, and bounded JWKS cache refresh behavior.

### Modified Capabilities

None.

## Impact

- Affects `src/presentation/extractor/claims.rs`, its error responses and logging, and authentication-focused unit or end-to-end tests.
- Does not change database or general GraphQL error handling, cache TTL policy, dependencies, or successful GraphQL authentication behavior.

## Context

The Claims extractor decodes bearer tokens, loads provider keys through a shared Moka cache, and validates JWT claims. Its JWKS fetch currently does not reject non-success HTTP status codes before decoding the body, and `ClientError::JwksFetch(String)` sends the stored internal error text directly to clients. The cache intentionally deduplicates concurrent misses and performs one invalidation and retry when a token key ID is absent.

## Goals / Non-Goals

**Goals:**

- Preserve a stable `401` boundary for client-controlled authentication failures.
- Return a safe, fixed `503` response for server configuration and JWKS infrastructure failures.
- Preserve detailed failure causes in one server-side log entry without logging tokens or authorization headers.
- Verify Claims extraction and bounded key-refresh behavior at its HTTP boundary.

**Non-Goals:**

- General database or GraphQL error handling changes.
- Changes to JWKS cache TTL, cache keys, or successful authentication contracts.
- Automatic retries beyond the existing single refresh for an unknown `kid`.

## Decisions

1. `ClientError::JwksFetch` will retain an `anyhow::Error` rather than a display-ready string. `IntoResponse` will log that cause and always emit the same public `server_error` description. This keeps the error chain available for diagnosis while preventing accidental disclosure. Logging at response conversion produces one error event per failed request and avoids duplicate logs in lower layers.
2. `fetch_jwks` will attach distinct context to client construction, request transport, HTTP status, and JSON decoding failures. It will call `error_for_status` before parsing JSON so 4xx/5xx responses are classified by status rather than body shape.
3. Server-configured URL and issuer construction failures remain JWKS/service failures because clients cannot correct them. Token parsing, key lookup, algorithm selection, signature, expiry, audience, and issuer validation failures remain authentication failures.
4. The extractor will retain the existing cache algorithm: cached lookup, one invalidation when `kid` is absent, one deduplicated reload, then `401` if still absent. Tests will count local JWKS server requests to prove both cache reuse and the refresh bound.
5. Tests will use the existing Axum/local-server authentication infrastructure where practical. Shared helpers and table-driven cases will limit duplication while assertions focus on status and public error fields rather than library-specific text.

## Risks / Trade-offs

- [Logging at `IntoResponse` loses an earlier layer-specific event name] → Preserve structured error context for each stage in the error chain and log it with debug formatting once.
- [Global `JWKS_URL` makes parallel unit tests interfere] → Prefer the existing serialized end-to-end setup or inject/localize test state if the current infrastructure supports it.
- [Exact JSON snapshots can make tests brittle] → Assert the stable public contract and the absence of representative internal details.
- [Changing the error payload could break consumers relying on leaked details] → Keep existing status, `error`, and `message` fields and only replace unsafe descriptions with the documented fixed description.

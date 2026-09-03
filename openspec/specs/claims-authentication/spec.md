# claims-authentication Specification

## Purpose
Define safe authentication failure responses, diagnosable JWKS infrastructure failures, and bounded JWKS cache refresh behavior for Claims extraction.
## Requirements
### Requirement: Authentication failures remain unauthorized
The system SHALL return `401 Unauthorized` for missing or malformed bearer authorization, undecodable JWT headers, missing or unknown key IDs, unsupported signing algorithms, and JWT validation failures including invalid signature, expiry, audience, and issuer.

#### Scenario: Authorization header is absent
- **WHEN** a request requiring Claims has no Authorization header
- **THEN** the system returns `401 Unauthorized` without panicking

#### Scenario: Token cannot be authenticated
- **WHEN** a bearer token is malformed, lacks a key ID, uses an unknown key or unsupported algorithm, or fails signature or registered-claim validation
- **THEN** the system returns `401 Unauthorized` with the public invalid-token contract and without JWT library details

### Requirement: JWKS infrastructure failures are unavailable
The system SHALL return `503 Service Unavailable` when it cannot construct the HTTP client, validate server-side JWKS configuration, connect to or time out while requesting the JWKS endpoint, receives an HTTP 4xx or 5xx response, or cannot decode a successful response as a JWKS document.

#### Scenario: JWKS endpoint returns an error status
- **WHEN** the configured JWKS endpoint returns an HTTP 4xx or 5xx response
- **THEN** the system treats the response as a JWKS fetch failure before attempting to decode its body and returns `503 Service Unavailable`

#### Scenario: JWKS endpoint returns invalid JSON
- **WHEN** the configured JWKS endpoint returns HTTP 200 with a body that is not valid JWKS JSON
- **THEN** the system returns `503 Service Unavailable`

#### Scenario: JWKS request or configuration fails
- **WHEN** a request cannot reach the endpoint or the server-configured JWKS URL or issuer domain is invalid
- **THEN** the system returns `503 Service Unavailable`

### Requirement: Infrastructure details remain private
The system MUST retain and log the internal cause of each JWKS infrastructure failure and SHALL return only a fixed safe public response containing `error` equal to `server_error`, `error_description` equal to `Authentication service is temporarily unavailable`, and `message` equal to `Service temporarily unavailable`.

#### Scenario: Internal JWKS error is rendered
- **WHEN** a JWKS infrastructure failure is converted into an HTTP response
- **THEN** the server logs the internal cause without the JWT or Authorization header and the response contains none of the transport, URL parsing, HTTP status, or JSON decoder details

### Requirement: JWKS cache and bounded refresh are preserved
The system SHALL normally use cached JWKS data, SHALL deduplicate concurrent cache-miss loads, and SHALL perform at most one cache invalidation and reload when a token key ID is absent from the cached JWKS.

#### Scenario: Cached key is available
- **WHEN** repeated authenticated requests use a key present in cached JWKS data
- **THEN** the system validates them without fetching JWKS for every request

#### Scenario: Rotated key appears after refresh
- **WHEN** a token key ID is absent from cached JWKS data and appears in the JWKS returned after invalidation
- **THEN** the system authenticates the token after exactly one refresh

#### Scenario: Key remains absent after refresh
- **WHEN** a token key ID is absent both before and after one cache invalidation and reload
- **THEN** the system returns `401 Unauthorized` and performs no further JWKS fetches for that extraction attempt

### Requirement: Existing JWKS URL safety policy is preserved
The system SHALL allow HTTPS JWKS URLs and loopback HTTP URLs, including localhost, IPv4 loopback, and IPv6 loopback, and SHALL reject non-loopback HTTP JWKS URLs as server-side configuration failures.

#### Scenario: Loopback and production URLs are evaluated
- **WHEN** the server validates a loopback HTTP JWKS URL, an HTTPS JWKS URL, or a non-loopback HTTP JWKS URL
- **THEN** it accepts the loopback and HTTPS URLs and rejects the non-loopback HTTP URL

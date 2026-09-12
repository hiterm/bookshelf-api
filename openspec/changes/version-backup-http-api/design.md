## Context

Backup is the first authenticated non-GraphQL JSON API. Its current unversioned routes also prescribe browser attachment filenames.

## Goals / Non-Goals

**Goals:** establish `/v1`, return ordinary JSON, and preserve the backup body and security behavior.

**Non-Goals:** GraphQL changes, compatibility routes, streaming, compression, or backup format changes.

## Decisions

- Nest a small backup router under `/v1`; `/graphql` remains unchanged.
- Remove `Content-Disposition` and its CORS exposure. The frontend owns filenames.
- Treat `/v1` and body `version: 1` as independent HTTP and file-format versions.

## Risks / Trade-offs

- Old clients break immediately → coordinate the frontend PR before release.

## 1. Specification

- [x] 1.1 Define rolling `main` image publication and post-publication validation semantics

## 2. Workflow

- [x] 2.1 Reuse the existing production build and add traceable image metadata
- [x] 2.2 Publish only on `main` pushes with job-scoped package permission
- [x] 2.3 Pull and health-check the registry image after publication while preserving PR and TLS validation

## 3. Verification

- [x] 3.1 Run OpenSpec, workflow syntax, actionlint, and relevant repository checks

## 4. Specification lifecycle

- [x] 4.1 Sync the delta spec into canonical specifications and archive the completed change

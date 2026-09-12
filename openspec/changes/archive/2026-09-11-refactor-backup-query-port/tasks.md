## 1. Architecture contract

- [x] 1.1 Narrow `layers.md` to the explicit `infrastructure -> use_case::port`
  exception and prohibit other infrastructure-to-use-case dependencies
- [x] 1.2 Remove every `domain -> use_case` dependency

## 2. Backup query port

- [x] 2.1 Define `BackupQueryPort` and projection types under `use_case::port`
- [x] 2.2 Rename the PostgreSQL implementation to `PgBackupQuery`
- [x] 2.3 Map the port projection to external backup DTOs in the interactor
- [x] 2.4 Add focused interactor delegation and mapping tests

## 3. Verification and delivery

- [x] 3.1 Run formatting, clippy, unit, integration, and E2E verification
- [x] 3.2 Archive the change and sync the delta specification
- [ ] 3.3 Update PR #340, verify CI, and obtain CodeRabbit approval

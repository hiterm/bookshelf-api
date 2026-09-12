## 1. Specification

- [x] 1.1 Remove export-time reference revalidation from the backup-export requirement while preserving the consistent-snapshot guarantee

## 2. HTTP Contract Coverage

- [x] 2.1 Assert exact snapshot and full HTTP JSON schemas with representative normal-write values
- [x] 2.2 Verify both endpoints reject unauthenticated requests
- [x] 2.3 Verify `exportedAt` and the attachment filename encode the same second

## 3. Validation

- [x] 3.1 Run backup unit, integration, and E2E validation

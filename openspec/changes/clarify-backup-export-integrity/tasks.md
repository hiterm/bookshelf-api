## 1. Specification

- [ ] 1.1 Remove export-time reference revalidation from the backup-export requirement while preserving the consistent-snapshot guarantee

## 2. HTTP Contract Coverage

- [ ] 2.1 Assert exact snapshot and full HTTP JSON schemas with representative normal-write values
- [ ] 2.2 Verify both endpoints reject unauthenticated requests
- [ ] 2.3 Verify `exportedAt` and the attachment filename encode the same second

## 3. Validation

- [ ] 3.1 Run backup unit, integration, and E2E validation

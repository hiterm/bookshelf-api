## 1. Contract

- [ ] 1.1 Commit OpenSpec separately and define version 1 DTOs

## 2. Repository and use case

- [ ] 2.1 Add a dedicated owner-scoped backup repository
- [ ] 2.2 Read both scopes in a read-only repeatable-read transaction
- [ ] 2.3 Map relations, validate references, and preserve stable order
- [ ] 2.4 Cover fidelity, history, baseline, edge cases, and tenant isolation

## 3. HTTP API

- [ ] 3.1 Add authenticated handlers and routes with matching UTC timestamps
- [ ] 3.2 Add handler and endpoint E2E coverage

## 4. Delivery

- [ ] 4.1 Run all repository checks and tests
- [ ] 4.2 Sync specs, archive, and commit archive separately
- [ ] 4.3 Create PR, verify CI, and obtain CodeRabbit approval

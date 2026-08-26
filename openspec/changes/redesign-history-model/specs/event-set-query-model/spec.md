## REMOVED Requirements

### Requirement: Unified event-set GraphQL type
**Reason**: The EventSet GraphQL model is replaced by the Operation model.

**Migration**: Clients query `operations` or `operation` and select Book/Author changes.

### Requirement: Selection-driven nested event loading
**Reason**: Nested entity Events no longer exist in the final GraphQL schema.

**Migration**: Operation nested changes preserve selection-driven loading.

### Requirement: Batched nested event loading
**Reason**: EventSet-to-Event loading is replaced by Operation-to-OperationChange loading.

**Migration**: Use user-scoped batched Book and Author change fields on Operation.

### Requirement: Scalar-only use-case event-set model
**Reason**: Event query use cases and EventSet DTOs are removed.

**Migration**: Use scalar Operation DTOs plus dedicated batched change operations.


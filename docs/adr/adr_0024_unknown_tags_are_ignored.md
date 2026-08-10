# ADR 0024: unknown tags are ignored

## Decision

Required tags occur exactly once with exact cardinality. Durable carriers reject
`expiration` and `-`. Every other unknown tag is ignored for draft-v1 state.

## Consequences

Extensions cannot change identity, authorization, dependencies, checkpoint
binding, or canonical output without a new version or kind.

# ADR 0021: exact base closure

## Decision

`base_heads` is a frontier. A child epoch begins from its exact accepted
ancestor closure within parent state. Frontier and closure are separate values.

## Consequences

Accepted ancestors survive epoch transitions, while parent changes outside the
selected closure become excluded from the child epoch.

# ADR 0026: fully metered panic-free evaluation

## Decision

Every deterministic traversal influenced by evidence charges typed work and
checks cancellation at deterministic boundaries. Evidence cannot reach
panic-only assertions or recursion proportional to input depth.

## Consequences

Capacity interruption returns an incomplete report; adapter and invariant
failures return typed noncanonical errors.

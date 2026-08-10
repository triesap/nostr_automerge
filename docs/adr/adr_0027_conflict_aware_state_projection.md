# ADR 0027: conflict-aware state projection

## Decision

Materialized paths carry deterministic conflict-branch identity. Projection
preserves exact scalar/object/text values and mark name, value, UTF-16 range,
and expansion semantics.

## Consequences

Nested conflicting object descendants cannot collapse onto one ordinary path,
and ambiguous assertions fail rather than select the first match.

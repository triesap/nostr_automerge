# ADR 0013: public engine API

## Decision

Export one documented batch engine from raw signed events through immutable
evidence to canonical controls, dispositions, heads, and materialized state.
Production callers cannot supply validation outcomes or internal index records.

## Consequences

Synthetic builders remain test-only. Conformance must use the public engine.

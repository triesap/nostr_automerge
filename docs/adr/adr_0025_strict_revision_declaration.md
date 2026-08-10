# ADR 0025: strict revision declaration

## Decision

Revision and profile probing is bounded, duplicate-aware, and canonical.
Malformed, duplicate, ambiguous, out-of-range, or noncanonical declarations are
invalid; only one canonical unknown declaration is unsupported.

## Consequences

Unsupported classification cannot bypass strict JSON validation.

# ADR 0062: Interrupted Finalization Settlement

## Status

Approved for remediation v8.

## Decision

Consume each actual partial-report pass while its finalization permit remains
active and forfeit only reservation proven unused.

## Rationale

Forfeiture represents unused capacity. It cannot truthfully account for
collection, digest, evidence, or invariant work performed after settlement.

## Consequences

- Report preparation is separated from terminal permit closure.
- Every dimension satisfies `reserved = consumed + refunded + forfeited`.
- The no-progress interrupted fallback remains constant.
- No wire format, event kind, public API, or hash domain changes.

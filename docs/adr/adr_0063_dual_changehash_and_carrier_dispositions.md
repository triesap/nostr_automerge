# ADR 0063: Dual ChangeHash And Carrier Dispositions

## Status

Approved for remediation v8.

## Decision

Emit aggregate `ChangeHash` outcomes and independent generic `Event` outcomes
for every attributable change carrier.

## Rationale

Semantic-state deduplication and signed-carrier accountability are distinct
protocol concerns. One semantic hash can have carriers with different branch,
authorization, revision, or payload outcomes.

## Consequences

- Every reportable change carrier has one Event record.
- Every semantic change hash has one aggregate ChangeHash record.
- Both record sets participate in canonical ordering and digest calculation.
- The existing generic Event namespace is reused without an unnecessary schema
  version change.

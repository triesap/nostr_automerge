# ADR 0061: Coordinate-Scoped Control Indexes

## Status

Approved for remediation v8.

## Decision

Preserve direct coordinate-qualified control membership and add direct
coordinate-qualified parent-edge, raw-change, and checked work indexes.

## Rationale

Filtering global controls or rebuilding global raw-change state during target
evaluation violates target work isolation even when the final output is later
filtered correctly.

## Consequences

- Target preparation and ancestry consume scoped borrowed data.
- Unrelated control evidence consumes no target budget or allocation.
- Existing coordinate indexes are extended rather than duplicated.
- No wire format, event kind, public API, or hash domain changes.

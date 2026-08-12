# ADR 0042: Global ChangeHash Carrier Claims

Status: Approved

## Context

Per-control candidate reconstruction can omit unresolved claims or allow a bad
duplicate claim to overwrite valid historical state.

## Decision

One semantic change is represented per coordinate and `ChangeHash`. Every
signed carrier is retained as a separate dynamically classified control claim,
and final hash state is reduced against the final canonical lineage.

## Consequences

One valid claim is sufficient, accepted-base hashes are not re-admitted, and
every attributable carrier has one represented hash outcome.

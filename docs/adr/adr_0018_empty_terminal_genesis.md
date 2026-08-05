# ADR 0018: empty terminal genesis

## Decision

A structurally valid genesis with no members is valid only as a terminal
control. No child control or change may extend it.

## Consequences

The implementation-only nonempty genesis ACL rule is removed.

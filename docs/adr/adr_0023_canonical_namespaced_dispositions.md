# ADR 0023: canonical namespaced dispositions

## Decision

Canonical disposition records distinguish control EventIds, ChangeHashes, and
other signed EventIds. Completion is limited to complete, budget exhausted, or
cancelled; unexpected failures return a typed noncanonical error.

## Consequences

Invalid, excluded, pending, accepted, and unsupported outcomes remain distinct,
and `Completion::Failed` is removed.

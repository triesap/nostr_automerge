# ADR 0015: checkpoint carrier authorization

## Decision

Checkpoint descriptors and chunks are accepted only as verified signed
carriers authorized by canonical control state and bound to one author and
descriptor identity.

## Consequences

Checkpoint primitives reproduce history but never authorize it.

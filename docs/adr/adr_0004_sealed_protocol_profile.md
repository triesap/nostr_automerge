# ADR 0004: sealed protocol profile

## Decision

Kinds, limits, encodings, ActorId derivation, selection rules, and semantics
are private constants owned by a sealed ProtocolRevision.

## Rejected

Caller-supplied ProtocolKinds/ProtocolLimits or feature flags that change
validity.

## Rationale

Customization would produce incompatible implementations claiming one NIP.

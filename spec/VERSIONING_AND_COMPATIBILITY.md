# Versioning and compatibility

## Protocol revision

The protocol profile is sealed and explicit.

Draft example:
`ProtocolRevision::Draft2026_08`

Final example:
`ProtocolRevision::V1`

No implicit fallback across revisions.

## Compatibility rules

A protocol revision controls:
- kinds;
- tags/content;
- actor derivation;
- Automerge profile;
- limits;
- selection/equivocation;
- digests;
- checkpoint rules.

Changing any of these requires a new revision and fixtures.

## Unsupported versus invalid

- unknown revision/profile: unsupported_revision;
- declared v1 with unknown or forbidden semantics: invalid.

## Crate semver

The crate may release pre-1.0 alpha versions while the NIP is draft.

Wire behavior changes require:
- spec change;
- ADR;
- fixture update;
- Rust and TypeScript agreement;
- release notes.

Public Rust API changes follow Cargo semver independently from wire revisions.

## Draft to final migration

When kinds/identifier are allocated:
- add final sealed revision;
- keep draft support only through explicit opt-in if needed;
- never accept draft kinds as final v1;
- update fixtures and NIP PR;
- document migration/successor strategy.

## Persistence compatibility

The initial core has no database schema. Later stores preserve raw signed
evidence and version their derived indexes independently.

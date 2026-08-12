# Conformance contract

## Principle

The prose specification is durable intent. Language-neutral fixtures make it
executable. Rust is a reference implementation, not the specification itself.

## Fixture families

- actor_derivation
- nip01_valid
- nip01_invalid
- canonical_json
- base64
- automerge_framing
- automerge_semantics
- manifest
- control_genesis
- control_transition
- control_fork
- change_authorization
- dependency_recovery
- device_equivocation
- randomized_delivery
- checkpoint_verified_history
- resource_budget
- versioning

## Raw artifacts

Use raw files for malformed data:
- `.json` for duplicate members and exact source;
- `.bin` for Automerge chunks;
- `.txt` for invalid base64/UTF-8 representations where suitable.

Do not store only already-parsed values.

## Expected report

Per fixture:
- fixture ID/revision;
- canonical controls;
- event-level and change-level dispositions;
- accepted/pending/excluded/invalid/unsupported sets;
- heads;
- history_digest;
- dispositions_digest;
- integrity alerts;
- typed state assertions;
- local completion expected only where fixture tests budgets.

## History digest

Before implementation consumes it, fix an exact domain-separated binary
encoding containing:
- protocol revision;
- coordinate;
- control IDs in chain order;
- accepted ChangeHashes sorted by bytes;
- heads sorted by bytes;
- fixed-width count encodings.

The exact formula lives in the fixture contract and requirements registry.

## Dispositions digest

Fix a domain-separated encoding over canonical item identifiers and their
protocol dispositions. Local completion is excluded.

## Typed assertions

Assertions preserve Automerge semantics rather than lossy JSON:
- null;
- bool;
- signed/unsigned integer;
- f64 exact bits;
- string scalar;
- text;
- bytes;
- timestamp;
- counter;
- list/map/table object type;
- marks;
- conflicts.

## Permutations

Every complete scenario is evaluated:
- fixture order;
- reverse;
- seeded permutations;
- duplicate-heavy;
- dependencies last;
- controls last;
- invalid carrier before/after valid duplicate.

Complete canonical output must match.

## Independent implementation

TypeScript:
- independently written;
- consumes the same neutral fixtures;
- no Rust code/service/WASM binding;
- uses Automerge JS/WASM only for Automerge engine;
- canonical report compared byte-for-byte.

## Claim levels

- parser/profile qualification;
- core-profile conformance;
- checkpoint-profile conformance;
- full draft-v1 conformance;
- independently interoperable;
- production-qualified.

These claims are distinct.

## Remediation v4 signed coverage

The canonical signed distribution additionally covers change-before-control,
pending and invalid referenced controls, cross-control and accepted-base
duplicate carriers, invalid-claim non-poisoning, pruned prior dependencies,
unrelated-coordinate isolation, malformed manifest prevalidation attribution,
and exact interrupted-finalization boundaries. Complete scenarios run under all
required delivery permutations through the public engine. The independent
TypeScript implementation must produce byte-identical canonical reports.

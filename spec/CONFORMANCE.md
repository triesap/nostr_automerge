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

The repository tracks five independent, fail-closed claim levels in
`claim_levels.json`:

- implementation completion;
- signed public-engine conformance;
- independent interoperability;
- release readiness;
- publication authorization.

Passing one level does not silently pass another. In particular, local
implementation completion is not a conformance, interoperability, release, or
publication claim. Publication remains a separate human-authority decision
even after every technical release gate is green.

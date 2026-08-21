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
protocol dispositions. `ControlEvent(EventId)` records describe signed control
outcomes, `ChangeHash(ChangeHash)` records describe semantic Automerge change
outcomes, and `Event(EventId)` records describe signed manifest, checkpoint,
and change-carrier outcomes. Every attributable change carrier has one `Event`
record in addition to the aggregate `ChangeHash` record for its semantic
change. All three namespaces share the existing canonical ordering and digest
encoding. Local completion is excluded.

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

## Requirement evidence

Every canonical requirement row binds its exact authority text, applicability,
implementation assertion or signed fixtures, implementation candidate, and
artifact digest. Cross-language rows remain pending until an opaque attestation
binds an independently written implementation to the same signed distribution
and byte-identical complete canonical reports. Evidence validators fail closed
on missing or reordered rows, stale authority or candidate identities, generic
critical proofs, weakened fixture coverage, altered artifacts, and status
overclaim.

## Remediation v8 signed coverage

The canonical signed v9 distribution contains exactly 180 scenarios. It
preserves all 171 signed v8 fixtures byte-for-byte and adds the nine named
branch-local change, coordinate-scoped control, and carrier Event cases. Every
complete implementation run executes two independent processes and all eight
declared delivery permutations for every fixture. Canonical report bytes must
be stable within each implementation, byte-identical across the Rust and
independently written TypeScript implementations, and a deliberate mismatch
must be rejected.

The distribution manifest binds the reconciled local NIP, companion,
requirements registry, schemas, signed inputs, and expected canonical reports
by SHA-256. A locked or incomplete transition is not conformance evidence.

## Signed conformance v10

This section is approved staged candidate authority at
`companion_authority_installed`. Its 192-scenario conformance claim becomes
current only when the closed authority transition reaches
`distribution_complete`; the checksum-bound v9 distribution remains the
current conformance baseline before that stage.

`NCRDT-CONF-010`: The checksum-bound signed v10 distribution MUST contain
exactly 192 scenarios, including the corrected checkpoint expectations and new
carrier, interruption, and work-boundary cases. Both implementations MUST
execute all scenarios twice and under all eight delivery permutations with
byte-identical canonical output and deliberate mismatch rejection.

The distribution preserves all 180 v9 scenario identities and signed input
bytes. It authorizes exactly four checkpoint expected-report corrections and
adds exactly twelve signed scenarios in four groups of three: checkpoint
control precedence, independent carrier outcomes, no-progress interruption,
and target-work boundaries. The eight permutations are `canonical`,
`reverse`, `seed_0`, `seed_24301`, `duplicate_heavy`, `dependencies_last`,
`controls_last`, and `invalid_before_valid`.

Historical v9 evidence remains immutable and explicitly superseded. It is not
re-evaluated or relabelled as proof of changed live authority. A passing local
v10 result does not authorize NIP submission, event-kind allocation,
publication, release, deployment, or production qualification.

## Semantically exact proof catalog

This staged section becomes a passing-evidence claim only after the 148-row
registry, proof catalog, result artifacts, candidate identities, and required
opaque independent-implementation overlay are all installed and validated.

`NCRDT-EVIDENCE-006`: Every passing requirement row MUST bind to a semantically
matching exact signed fixture or named assertion through a validated proof
catalog. Broad command-only proof, unrelated assertion categories, stale
expectations, and missing opaque TypeScript evidence identifiers MUST be
rejected.

Each proof entry binds the exact requirement and authority text,
applicability, implementation candidate, semantic category, exact named
assertion or signed fixture, executed command, result artifact, and artifact
SHA-256. Cross-language rows additionally bind an opaque compatibility
evidence identity without importing source, paths, logs, URLs, workflows, or
runtime artifacts. Validators reject missing, duplicate, reordered, stale,
generic, category-mismatched, false-held, scope-leaking, or hash-mismatched
proof, and reject a passing assertion that did not execute.

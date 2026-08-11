# nostr_automerge_v1_spec

## Status

Approved implementation baseline.

The NIP text in `NIP_DRAFT.md` is a read-only snapshot of the externally
authored normative proposal. This companion specification records
implementation invariants, claim levels, and pressure-tested clarifications
that are required for this repository's conformance profile. It does not claim
that the external NIP prose was edited, reconciled, submitted, or adopted.

## Protocol thesis

```text
signed immutable Nostr evidence
+ exact Automerge Change Chunks
+ causal controller authorization
+ deterministic client evaluation
= relay-neutral local-first documents
```

## Core invariants

1. Same complete relevant evidence produces the same authorized history and
   document.
2. Transport and arrival order never affect canonical state.
3. Relay acceptance is not protocol validity.
4. Controller governance and device authorship are distinct.
5. Authorization transitions are causal frontiers, never timestamps.
6. Every device has one deterministic per-document ActorId.
7. One valid carrier is sufficient for a ChangeHash.
8. Device equivocation has no arbitrary winner.
9. Controller fork selection is deterministic and alerting.
10. The Automerge profile is exact and sealed.
11. Known-v1 unknown semantics are invalid.
12. Local resource refusal is not invalidity.
13. Checkpoints accelerate only fully verified history.
14. Batch replay is the initial reference oracle.
15. Rust and TypeScript fixtures are required before interoperability claims.

## Causal actor sequence and operation counter

Actor change sequence is actor-local. It starts at one and increases by exactly
one. For a candidate with sequence greater than one, its exact accepted
dependency closure contains exactly one same-actor change with the preceding
sequence.

The Automerge operation counter is causal rather than actor-local. For a
candidate change `C`, let `D(C)` be its exact accepted dependency closure:

```text
next_op(C) = 1                          when D(C) contains no operations
next_op(C) = 1 + max(operation_counter) otherwise
require C.start_op == next_op(C)
```

Equivalently, an implementation may take the maximum exclusive next-operation
value exposed by the changes in `D(C)`. An operation-bearing change advances
that value by its operation count. An empty change consumes one actor sequence
and does not advance the operation counter. Every addition and conversion is
checked for overflow. Only the exact accepted closure contributes; unrelated,
pending, excluded, invalid, or later changes do not.

## Selected manifest dynamic validity

The manifest's signed structure, canonical content, field values, and limits
remain exactly those defined by the read-only NIP snapshot. This section adds
the dynamic meaning of a statically valid selected manifest.

NIP-01 addressable replacement selection occurs before semantic validation.
Only the selected event is validated, and an unavailable or invalid selected
event never causes fallback to an older event.

The selected manifest's referenced control is then resolved against the
stateful control outcomes for the same document coordinate:

- an accepted canonical control makes the advisory hint canonical and
  available;
- a statefully valid but excluded noncanonical control makes the advisory hint
  noncanonical and available;
- a missing or pending control makes the manifest pending and unavailable;
- a wrong-kind, wrong-coordinate, unsupported, statically invalid, or
  dynamically invalid control makes the manifest invalid and unavailable.

A manifest is advisory. It never selects control history, grants
authorization, or establishes checkpoint trust.

## Dynamic signed-event dispositions

Static signed-carrier validity is ingress evidence status, not necessarily the
final protocol disposition. Every dynamically evaluated manifest, checkpoint
descriptor, and checkpoint chunk has exactly one canonical record in the
`event` namespace, and that record participates in the dispositions digest.

- A manifest is `accepted` when selected and dynamically valid, `excluded`
  when statically valid but not selected by replacement, `pending` when its
  selected control is missing or pending, `invalid` for known-v1 structural or
  dynamic failure, and `unsupported_revision` only for a unique canonical
  unknown revision or profile.
- A checkpoint descriptor is `accepted` only when complete, authorized,
  correctly assembled, and fully verified. It is `pending` while required
  control, chunk, or historical-carrier evidence is absent; `invalid` for a
  known-v1 authorization, binding, structure, commitment, snapshot, or history
  failure; and `unsupported_revision` only for a unique canonical unknown
  revision or profile.
- A checkpoint chunk is `accepted` only as a member of a fully verified
  descriptor set. It is `pending` while the descriptor or another required
  chunk is absent; `invalid` for a known-v1 author, coordinate, descriptor,
  count, index, content, proof, or verification mismatch; and
  `unsupported_revision` only for a unique canonical unknown revision or
  profile.

Verified checkpoints remain acceleration artifacts. They never authorize or
redefine document history.

## Implementation claim levels

### foundation

Workspace, sealed profile, semantic types, fixture loader, CI.

### automerge_qualified

Framing, UTF-16, migration policy, counters, canonical re-encoding and semantic
coverage pass.

### core_profile

Strict NIP-01, manifests, controls, changes, deterministic evaluation,
equivocation, reports and fixtures pass in Rust.

### independent_core_interop

Independent TypeScript implementation agrees on all required core fixtures.

### checkpoint_profile

Verified-history checkpoint fixtures pass.

### full_draft_v1

Core + checkpoints + independent interop + security/resource gates.

### production_qualified

External review, fixed limits, fuzzing/load evidence, release controls.

## Repository boundary

The standalone crate is protocol-level only. The Farm Workspaces product and
local-sync transports are preserved as downstream context but do not influence
generic API or validity.

## Approved implementation order

Follow `implementation/COMMIT_SEQUENCE.md`. It is an executable plan, not a
replacement for this specification.

## Change control

Any proposed change to accepted evidence, actor derivation, control selection,
Automerge semantics, canonical encoding, digests, checkpoint verification, or
protocol limits requires:
- ADR;
- requirement update;
- companion-spec update and separately tracked external NIP reconciliation;
- fixture update;
- Rust update;
- TypeScript update;
- differential and migration analysis.

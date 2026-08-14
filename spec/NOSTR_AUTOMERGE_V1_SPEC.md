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

## Remediation v4 execution rules

These implementation-owned rules are normative for this repository while the
external NIP prose is reconciled separately.

### Coordinate scope

Evaluation is performed for exactly one document coordinate. Canonical
controls, change and event dispositions, digests, checkpoint results, public
evidence, and local work accounting derive only from attributable target
evidence. Explicitly referenced predecessor or successor lifecycle evidence may
be read and charged as non-reportable support. Unattributable invalid raw bytes
and unrelated documents do not affect the target report or completion.

### Semantic changes and carrier claims

`ChangeHash` is the semantic identity across every target carrier. Each signed
carrier remains a distinct claim naming an event, control, author, and
coordinate. All carriers for one hash must expose identical canonical Change
Chunk bytes and semantic metadata; control and event identifiers are claim
metadata rather than semantic identity.

A claim naming a missing or pending control is pending. A claim naming a
wrong-kind, wrong-coordinate, statically invalid, dynamically invalid, or
unauthorized control is invalid. An otherwise-valid claim naming a statefully
valid noncanonical control is excluded. A canonical eligible claim participates
in epoch evaluation. One dynamically valid claim is sufficient, and other
invalid, pending, unsupported, or noncanonical claims cannot poison it.

A hash already in the selected accepted base is not a current-epoch candidate.
Final hash disposition is reduced against the final canonical lineage: final
accepted closure is accepted; state accepted at a canonical ancestor but pruned
from the final lineage is excluded; otherwise unresolved claims are pending,
noncanonical valid claims are excluded, all-unsupported claims are unsupported,
and remaining conclusive failures are invalid. Every attributable validated
change carrier yields exactly one represented hash outcome.

### Prior epoch knowledge

Epoch evaluation distinguishes selected accepted base, known canonical-ancestor
state outside that base, known invalid earlier state, same-epoch candidates,
and genuinely absent evidence. A dependency on known earlier state outside the
fixed signed base or known invalid earlier state is invalid. Truly absent or
unresolved evidence remains pending and may promote after delivery. Same-epoch
equivocation and descendants remain excluded through quarantine rather than
being recategorized as invalid dependencies.

### Manifest prevalidation attribution

Before full validation of a validly signed kind-31624 event, collect every
syntactically valid document-ID value from its `d` tags. If the set contains
exactly one distinct value, the event is attributable to that coordinate for
replacement ordering. Full validation still rejects missing, repeated,
malformed, or extra-element tags, and a selected invalid event suppresses older
fallback. Zero or multiple distinct valid values makes the event invalid and
unattributable.

### Reserved report finalization

Every evidence-proportional traversal, allocation, vector construction, and
digest encoding required after budget exhaustion or cancellation is planned
and atomically reserved before canonical state work. A failed reservation
returns a constant-size interrupted report before state evaluation and without
fabricated canonical progress. After a stop, optional work ceases and only
bounded mandatory finalization may consume the reservation.

## Remediation v5 execution rules

These implementation-owned rules refine the preceding sections while the
external NIP remains read-only and separately reconciled.

### Shared referenced-control resolution

Every manifest, change claim, dependency, and checkpoint resolves a referenced
control through the same state machine: canonical, statefully valid
noncanonical, pending, missing, wrong kind, wrong coordinate, statically
invalid, dynamically invalid, or unsupported. Missing and pending remain
recoverable. A known unusable referenced control makes a dependent draft-v1
carrier invalid; the dependent carrier does not inherit an unsupported revision.

Manifest hints are available for canonical and valid noncanonical controls,
pending for missing or pending controls, and unavailable for every known
unusable control. Change claims additionally require write authorization.
Checkpoint descriptors additionally require checkpoint authorization;
noncanonical and every known unusable control is invalid.

### Reasoned final ChangeHash reduction

Semantic change data, signed carrier metadata, per-claim reason, and final
lineage state are separate. A generic prior protocol disposition is not a claim
reason. For every target `ChangeHash`, apply the first matching rule:

1. final accepted closure is `accepted`;
2. a canonical ancestor accepted and later pruned is `excluded`;
3. any genuinely unresolved claim or selected-epoch dependency is `pending`;
4. any otherwise-valid noncanonical or current-branch excluded result is
   `excluded`;
5. a nonempty set containing only unsupported claims is
   `unsupported_revision`;
6. every remaining conclusive known-v1 failure is `invalid`.

Accepted state cannot be poisoned, and canonical pruning outranks a later
pending duplicate.

### Complete dependency knowledge

For a selected signed epoch, classify every dependency as accepted in base,
same-epoch candidate, pruned canonical ancestor, known through another control,
known invalid, known unsupported, prior-equivocation-excluded, or unknown.
Accepted-base state is usable and same-epoch state is resolved in the selected
graph. Every other known state is impossible under that epoch and invalidates
the dependant transitively. Only genuinely absent or unresolved
selected-control evidence remains pending. A selected-control claim takes
same-epoch priority over duplicate claims naming other controls.

### Indexed coordinate work

Corpus finalization derives deterministic indexes for reportable events,
semantic hashes and claims, manifests, checkpoints, attributable invalid or
unsupported carriers, and direct lifecycle support by coordinate. Evaluation
checks cancellation before target lookup and does not scan unrelated documents.
Manifest selection iterates indexed candidates without cloning evidence.
Every target hash, claim, referenced-control lookup, role comparison, and
lifecycle-support access consumes deterministic local work capacity.

### Mechanically enforced finalization

Before interruptible canonical work, reserve checked conservative dimensions
for controls, changes, events, checkpoints, digests, evidence, invariant
validation, and fixed report overhead. Each finalization pass consumes its own
dimension. Cross-dimension borrowing, underflow, overrun, double consumption,
or unexplained remainder is a noncanonical invariant failure. Failed reservation
returns a constant-size no-progress interruption report. Completed evaluation
refunds unused optional capacity exactly once.

## Remediation v6 reconciliation authority

The following rules are the self-contained local authority mirrored by the
unsubmitted external patch proposal. They do not modify the read-only NIP
snapshot or any wire constant.

### Causal operation counters

For a candidate change `C`, let `D(C)` be its exact accepted dependency closure.
`next_op(C)` is one when `D(C)` contains no operations; otherwise it is one plus
the greatest operation counter visible in `D(C)`. `C.start_op` must equal
`next_op(C)`, with checked arithmetic and conversions. Actor sequence is a
separate actor-local rule: it starts at one, increments exactly, and an empty
change consumes sequence without advancing the causal operation counter.

### Coordinate-scoped evaluation

Evaluation for one coordinate derives all reportable controls, changes, event
dispositions, checkpoints, digests, evidence, output, and local work from
evidence attributable to that coordinate. Explicit predecessor or successor
lifecycle evidence may be read only as nonreportable support. Unattributable or
unrelated-coordinate evidence cannot affect the target report, completion, or
target work counters. Cancellation and capacity checks precede target lookup
and every target-proportional allocation.

### Semantic ChangeHash claims

`ChangeHash` is semantic change identity; each signed kind-1624 carrier is an
independent claim about its bytes, author, coordinate, and control. Evaluation
retains reasoned per-claim state, groups every attributable carrier by semantic
hash, and admits a hash when any claim is dynamically valid. Invalid, pending,
unsupported, unauthorized, terminal-control, or noncanonical claims cannot
poison a valid claim. A hash already in the selected accepted base is not
readmitted as a new epoch candidate.

### Dependent change authorization

A draft-v1 change whose referenced control is wrong-kind, wrong-coordinate,
statically invalid, dynamically invalid, or unsupported is invalid; it does not
inherit `unsupported_revision`. After resolving a valid control, compare the
signed device, derived ActorId, and `write` role before considering canonical
versus noncanonical branch disposition. An unauthorized noncanonical claim is
invalid, and a terminal control authorizes no change.

### Final claim precedence

Reduce each target `ChangeHash` by the first matching rule: final accepted
closure is `accepted`; a canonical ancestor accepted then pruned is `excluded`;
any genuinely unresolved claim is `pending`; an otherwise-valid noncanonical
or current-branch-excluded claim is `excluded`; a nonempty claim set containing
only unsupported carriers is `unsupported_revision`; every remaining conclusive
failure is `invalid`. Accepted and canonical-pruned lineage outrank later
duplicate claims, while pending outranks noncanonical and conclusive failures.

### Complete dependency knowledge

For each selected signed epoch, classify a dependency as accepted in base,
same-epoch candidate, pruned canonical ancestor, known through another control,
known invalid, known unsupported, prior-equivocation-excluded, or unknown.
Accepted-base dependencies are usable and same-epoch candidates resolve in the
selected graph. Every other known state is impossible under that epoch and
invalidates the dependant and its descendants. Only genuinely absent or
unresolved selected-control evidence remains pending and may promote.

### Control parent and frontier references

Resolve a child control parent as canonical, valid noncanonical, pending,
missing, wrong kind, wrong coordinate, statically invalid, dynamically invalid,
or unsupported. Missing or pending makes the child pending; every known
unusable state makes it invalid. Validate a valid noncanonical child relative
to its own ancestry before exclusion. For each base head, accepted-under-parent
is usable, missing or pending is pending, and invalid-under-parent,
excluded-under-parent, unsupported, or other-control is invalid. Pending and
invalid ancestry propagate their respective state to descendants.

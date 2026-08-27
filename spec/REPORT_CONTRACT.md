# Evaluation report and interruption contract

## Status and authority

This document is approved authority for the staged local remediation-v9
implementation candidate at `companion_authority_installed`. It does not edit
or override the repository-local NIP draft. Where this candidate contract and
the unchanged NIP disagree, the NIP continues to control NIP-conformance
claims. Candidate closure, NIP conformance, publication, release, deployment,
and production qualification remain held.

The typed Rust report may add a `ProtocolRevision` member and typed local
failure without changing `nostr_automerge.report.v1`, a signed carrier, a wire
constant, a digest domain, or the sealed protocol revision. The neutral report
keeps its existing `revision` member.

## Complete canonical report

A report whose completion is `complete` contains one exact, mutually
consistent view of its requested coordinate and sealed revision. It includes
the canonical controls, all three disposition namespaces, semantic change
partitions, change-carrier outcomes, heads, evidence, checkpoint outcomes,
manifest availability, integrity alerts, history and dispositions digests,
typed assertions, and materialized document state required by that
evaluation.

Every collection is complete, unique, disjoint where its type requires
disjointness, and strictly ordered by its declared byte-oriented rule. Every
digest is recomputed from the same report view. Every attributable verified
change carrier has exactly one `Event(EventId)` disposition record, and every
verified semantic change has exactly one `ChangeHash(ChangeHash)` disposition
record. `ControlEvent(EventId)`, `ChangeHash(ChangeHash)`, and
`Event(EventId)` remain disjoint namespaces.

## Independent change-carrier outcomes

`NCRDT-DISPOSITION-006`: A change-carrier Event disposition MUST be derived
from that carrier claim and its referenced control or branch. An aggregate
ChangeHash disposition MUST NOT convert a carrier with a known-invalid
reference into accepted, pending, or excluded.

Valid-carrier dominance applies only to the aggregate semantic outcome. It
does not hide or rewrite an invalid, pending, unsupported, or noncanonical
carrier Event. A semantic change is applied at most once even when several
carriers establish or claim it.

## No-progress interruption reports

`NCRDT-INTERRUPT-001`: A public evaluation that ends in `budget_exhausted` or
`cancelled` MUST return a constant-size no-progress report. It MUST NOT expose
canonical controls, protocol dispositions, evidence, checkpoints, an available
or resolved manifest, integrity alerts, heads, or materialized document state.

The no-progress report contains the requested coordinate, the evaluator's
sealed revision, the incomplete completion, the matching typed local failure
where that API exposes it, and the canonical empty-input history and
dispositions digests for that coordinate and revision. Every canonical or
target-sized collection is empty, its canonical manifest availability is
`missing`, and the document is absent. The first typed stopping cause is
preserved; later cancellation observation cannot relabel budget exhaustion.
Reevaluation stops as soon as either input report is incomplete and performs
no summary, alert, or target-proportional comparison work.

## Two-tier finalization reservation

`NCRDT-RESOURCE-013`: The evaluator MUST reserve fixed no-progress fallback
capacity separately from complete-report capacity. Actual complete-report
passes are consumed immediately before their work; on interruption,
complete-report capacity is forfeited and only fixed fallback passes are
consumed.

The fixed fallback ledger is independent of caller-selected target capacity
and is sufficient at zero caller budget. The complete ledger is planned from
retained target metadata before interruptible canonical work. Its named passes
cover control records, semantic `ChangeHash` records, change-carrier Events,
other Events, checkpoint records, change classifications, history-digest
input and encoding, dispositions-digest input and encoding, evidence records,
report invariants, and fixed complete-report overhead.

The two ledgers cannot borrow, transfer, refund, or disguise capacity between
one another. Each pass consumes its stable dimension and amount immediately
before its owned work. On interruption, unperformed complete capacity is
forfeited before the fixed fallback is constructed and both ledgers close
exactly once. On completion, the complete report and its invariants are
validated before proven unused complete capacity is refunded. In every
dimension:

```text
reserved = consumed + refunded + forfeited
```

Overflow, underflow, borrowing, duplicate or reordered consumption, wrong-pass
consumption, early refund, double closure, and unexplained remainder are typed
noncanonical invariant failures.

## Target-local deterministic work

`NCRDT-RESOURCE-014`: Every target-proportional preparation collection,
raw-byte copy or shared-reference operation, branch memo traversal, canonical
derivation pass, alert copy, and disposition copy MUST be bounded, charged,
cancellation-aware, or eliminated.

The same ownership rule covers target controls, parent and dependency edges,
members, change hashes, carriers, checkpoint joins, proof visits, Automerge
decode and application, projection units, sorting, comparisons, evidence,
digests, reports, and invariant traversals. Cancellation is checked before
target lookup, before target-proportional allocation, and during every
proportional traversal. All counts and conversions use checked arithmetic.

Canonical raw change bytes are retained once in shared immutable storage
through the Rust target path. Cloning a shared reference is an item charge; a
real byte copy is charged by bytes. Reportable state and charged target work
come from coordinate-qualified membership, so unrelated evidence cannot
change output, completion, allocations, copies, or work counters.

### Metered persistent-state operations

`NCRDT-RESOURCE-015`: Every runtime lookup, membership test, extension, or
materialization over persistent branch state MUST charge and check cancellation
before each visited persistent node and each inserted target item, or use a
separately metered flattened representation.

### No target-sized work after a stop

`NCRDT-RESOURCE-016`: After a work-budget charge fails or cancellation is
observed, evaluation MUST perform no further target-sized traversal,
allocation, copy, comparison, serialization, or invariant construction and
MUST return the constant-size no-progress result.

## Unsupported change identity

`NCRDT-VERSION-002`: An unsupported change carrier whose canonical Change
Chunk and ChangeHash were not verified receives only an Event
`unsupported_revision` outcome. Its unverified `x` tag MUST NOT create a
semantic ChangeHash disposition in draft v1.

The unverified tag also cannot create dependency identity, accepted state, a
head, or aggregate-reducer input. If supported canonical bytes are later
verified and their ChangeHash is computed under the sealed profile, the normal
independent carrier-Event and semantic-ChangeHash rules apply.

## Construction and parsing invariants

Every complete/no-progress builder, report-parts constructor, parser,
serializer, fixture loader, reevaluation consumer, and public getter preserves
and validates the report revision. Complete reports require exact semantic
partitions, coverage, ordering, controls, carrier consistency, checkpoint and
manifest consistency, evidence, alerts, state, and recomputed digests.

Constructors and parsers reject duplicate, unsorted, overlapping, extra,
missing, inconsistent, or digest-mismatched input. They do not sort,
deduplicate, repair, infer, or fill canonical report data. Internal partial
evaluator state is never a public `EvaluationReport`. Internal invariant,
graph, adapter, decode, apply, or projection failures remain typed
noncanonical errors rather than protocol dispositions.

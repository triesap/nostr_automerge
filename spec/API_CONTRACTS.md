# API contracts

This document defines intended public API responsibilities. Exact Rust syntax
may evolve through implementation review, but behavior and boundaries are
durable.

## Design rules

- accept raw NIP-01 JSON at the trust boundary;
- expose semantic newtypes, not interchangeable strings;
- expose owned immutable evidence/report types;
- do not expose mutable Automerge state in the first public API;
- do not expose third-party Nostr/Automerge errors or types as the stable API;
- no async functions;
- no I/O;
- no global state.

## Sealed revision

```rust
#[non_exhaustive]
pub enum ProtocolRevision {
    Draft2026_08,
}
```

A caller can select a supported revision. It cannot provide custom kinds,
limits, or algorithms.

## Document coordinate

```rust
pub struct DocumentCoordinate {
    controller: ControllerPublicKey,
    document_id: DocumentId,
}
```

Responsibilities:
- strict construction;
- canonical NIP-01 address rendering;
- byte-oriented equality/order;
- no relay hints.

## Corpus construction

```rust
pub struct CorpusBuilder { /* private */ }

impl CorpusBuilder {
    pub fn new(revision: ProtocolRevision) -> Self;
    pub fn ingest(&mut self, raw_event_json: &[u8]) -> IngestOutcome;
    pub fn finish(self) -> EvidenceCorpus;
}
```

`ingest`:
- bounds raw input;
- performs strict parsing and verification;
- preserves raw evidence;
- never evaluates the whole document;
- returns event-level diagnostic outcome;
- is idempotent by EventId.

## Evaluation

The signature below records the staged remediation-v9 candidate semantics. It
becomes local implementation authority at `companion_authority_installed`
without editing or overriding the unchanged repository-local NIP draft.

```rust
pub struct ReferenceEvaluator { /* private */ }

impl ReferenceEvaluator {
    pub fn new(revision: ProtocolRevision) -> Self;

    pub fn evaluate(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<EvaluationReport, EvaluationError>;
}
```

No wall-clock deadline is part of evaluation. Equal immutable evidence,
coordinate, revision, budget, and cancellation boundary produce the same
result. Typed invariant, graph, adapter, decode, apply, or projection failures
are noncanonical `EvaluationError`s rather than protocol dispositions.

## Work budget

```rust
pub struct WorkBudget {
    pub max_events_examined: u64,
    pub max_graph_nodes_examined: u64,
    pub max_graph_edges_examined: u64,
    pub max_bytes_decoded: u64,
    pub max_change_applications: u64,
}
```

These are local execution controls, not protocol validity limits.

## Protocol disposition

```rust
pub enum ProtocolDisposition {
    Accepted,
    Pending,
    Excluded,
    Invalid,
    UnsupportedRevision,
}
```

## Carrier Event and semantic ChangeHash identities

The report preserves two independent identity layers. Every attributable
change carrier receives one Event outcome derived from that carrier's signed
revision, payload binding, control reference, authorization, and branch-local
result. A semantic `ChangeHash` outcome exists only after supported canonical
change bytes have been verified and their hash computed under the sealed
profile. Aggregate semantic reduction never rewrites a known-invalid carrier
Event.

An unsupported carrier whose canonical change bytes and hash were not verified
remains visible as an Event with `unsupported_revision`. Its unverified `x` tag
does not create a semantic disposition, dependency identity, accepted-state
entry, head, or aggregate-reducer input.

## Local completion

```rust
pub enum EvaluationCompletion {
    Complete,
    BudgetExhausted,
    Cancelled,
}
```

Local completion never changes canonical disposition.

## Evaluation report

```rust
pub struct EvaluationReport {
    pub revision: ProtocolRevision,
    pub coordinate: DocumentCoordinate,
    pub canonical_controls: Vec<EventId>,
    pub accepted_changes: Vec<ChangeHash>,
    pub pending_changes: Vec<ChangeHash>,
    pub excluded_changes: Vec<ChangeHash>,
    pub invalid_events: Vec<EventId>,
    pub unsupported_events: Vec<EventId>,
    pub heads: Vec<ChangeHash>,
    pub history_digest: HistoryDigest,
    pub dispositions_digest: DispositionsDigest,
    pub integrity_alerts: Vec<IntegrityAlert>,
    pub completion: EvaluationCompletion,
}
```

All vectors are in their specified canonical order. The displayed fields are
illustrative rather than an exhaustive constructor: the report also preserves
the complete disposition namespaces, per-carrier Event outcomes, checkpoint
and evidence views, manifest availability, typed local failure where exposed,
and optional materialized document required by
[`REPORT_CONTRACT.md`](REPORT_CONTRACT.md).

A `complete` report exposes one exact canonical protocol view. A
`budget_exhausted` or `cancelled` report exposes only the constant-size,
revision-bound no-progress shape and empty-domain digests. Public construction
and parsing reject duplicate, unsorted, overlapping, extra, missing, repaired,
or cross-inconsistent report data.

Materialized state may be exposed through:
- read-only query methods;
- typed assertion evaluation;
- an opaque document view.

The initial stable API should not hand out `&mut automerge::Automerge`.

## Authoring primitives

Added only after evaluator and conformance stabilize.

Authoring APIs:
- derive ActorId;
- construct canonical genesis/control content;
- create canonical Automerge change with fixed metadata;
- return unsigned carrier draft;
- accept caller-supplied created_at and signing boundary;
- return explicit actor-state transition.

The core does not persist keys, sign with platform key stores, publish, or
manage outboxes.

## Error contract

Errors are typed and stage-specific:
- RawEventError
- Nip01VerificationError
- ContentCanonicalizationError
- CarrierValidationError
- AutomergeFramingError
- AutomergeSemanticError
- ControlValidationError
- EvaluationError
- CheckpointError

Every error:
- is deterministic for the same bytes and revision;
- has stable machine-readable code;
- avoids raw content in Display/Debug by default;
- preserves diagnostic source internally where safe;
- does not use error strings as protocol logic.

Local `EvaluationFailure` values, when the report API exposes them, correspond
exactly to `budget_exhausted` or `cancelled`. They do not encode protocol
invalidity, and later cancellation checks cannot replace the first typed stop
cause.

## Compatibility

Before `1.0`, the public API may change under semver pre-release rules.
Canonical wire behavior and fixture outputs change only through an approved
protocol revision/ADR.

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

```rust
pub struct ReferenceEvaluator { /* private */ }

impl ReferenceEvaluator {
    pub fn new(revision: ProtocolRevision) -> Self;

    pub fn evaluate(
        &self,
        coordinate: &DocumentCoordinate,
        corpus: &EvidenceCorpus,
        budget: &WorkBudget,
        cancellation: &dyn CancellationCheck,
    ) -> EvaluationReport;
}
```

No wall-clock deadline is part of evaluation.

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

All vectors are in their specified canonical order.

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

## Compatibility

Before `1.0`, the public API may change under semver pre-release rules.
Canonical wire behavior and fixture outputs change only through an approved
protocol revision/ADR.
